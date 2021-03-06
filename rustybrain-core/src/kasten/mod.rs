use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::{self, create_dir_all, DirEntry},
    path::{Path, PathBuf},
    rc::Rc,
    slice::Iter,
    time::SystemTime,
    usize,
};

use chrono::Local;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, TEXT},
    Document, Index,
};

use crate::{config::Config, zettel::Zettel};

#[derive(Clone)]
pub struct Kasten {
    config: Rc<RefCell<Config>>,

    #[allow(dead_code)]
    schema: Schema,

    title: Field,
    body: Field,
    path: Field,
    index: Index,

    zettels: Vec<Rc<RefCell<Zettel>>>,
    backlinks: HashMap<String, Vec<usize>>,
}

impl Kasten {
    pub fn new(config: Rc<RefCell<Config>>) -> Result<Self, anyhow::Error> {
        let mut schema_builder = Schema::builder();
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let path = schema_builder.add_text_field("path", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        let mut kasten = Kasten {
            config,
            schema,
            index,
            title,
            body,
            path,

            zettels: vec![],
            backlinks: HashMap::new(),
        };
        kasten.build()?;
        Ok(kasten)
    }

    fn build(&mut self) -> Result<(), anyhow::Error> {
        self.build_index()?;
        let mut zettels = vec![];
        let mut backlinks: HashMap<String, Vec<usize>> = HashMap::new();
        for entry in self.iter_from_disk() {
            let z = entry?;
            for link_to in z.link_to_iter() {
                if let Some(v) = backlinks.get_mut(link_to) {
                    v.push(zettels.len());
                } else {
                    backlinks.insert(link_to.to_string(), vec![zettels.len()]);
                }
            }
            zettels.push(Rc::new(RefCell::new(z)));
        }
        self.zettels = zettels;
        self.backlinks = backlinks;
        Ok(())
    }

    fn build_index(&self) -> Result<(), anyhow::Error> {
        {
            let mut index_writer = self.index.writer(50_000_000)?;
            index_writer.delete_all_documents()?;
            index_writer.commit()?;
        }

        for entry in self.zettels.iter() {
            let z = entry.borrow();
            self.add_doc(&z)?;
        }
        Ok(())
    }

    fn add_doc(&self, z: &Zettel) -> Result<(), anyhow::Error> {
        let mut index_writer = self.index.writer(50_000_000)?;
        let title = self.title;
        let body = self.body;
        let mut doc = Document::default();
        doc.add_text(title, z.title());
        doc.add_text(body, z.content());
        if let Some(p) = z.path().to_str() {
            doc.add_text(self.path, p);
        }
        index_writer.add_document(doc)?;
        index_writer.commit()?;
        Ok(())
    }

    pub fn search_title(
        &self,
        kw: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommit)
            .try_into()?;
        let searcher = reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.title]);
        let query = query_parser.parse_query(kw)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        let mut set = HashSet::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: Document = searcher.doc(doc_address)?;
            if let Some(Value::Str(s)) = retrieved_doc.get_first(self.path) {
                set.insert(s.to_string());
            }
        }
        Ok(set)
    }

    pub fn iter(&self) -> Iter<'_, Rc<RefCell<Zettel>>> {
        self.zettels.iter()
    }

    fn iter_from_disk(&self) -> SyncDiskIter {
        let c = (*self.config).borrow();
        SyncDiskIter {
            inner: None,
            repo_path: c.repo_path().to_string(),
        }
    }

    pub fn create(
        &mut self,
        title: &str,
    ) -> Result<Rc<RefCell<Zettel>>, anyhow::Error> {
        let path = self.new_path();
        if let Some(dir) = path.as_path().parent() {
            create_dir_all(dir)?;
        }
        let z = Zettel::create(&self.repo_path(), &path, title)?;
        self.add_doc(&z)?;
        let z = Rc::new(RefCell::new(z));
        self.zettels.push(z.clone());
        Ok(z)
    }

    pub fn save(&mut self, zettel: &Zettel) -> Result<(), anyhow::Error> {
        zettel.save()?;
        self.build()?;
        Ok(())
    }

    fn new_path(&self) -> PathBuf {
        let c = (*self.config).borrow();
        let path = c.repo_path();
        let gen = Local::now().format("%Y%m%d%H%M%S").to_string();
        Path::new(path).join(format!("notes/{}.md", gen))
    }

    pub fn repo_path(&self) -> String {
        self.config.borrow().repo_path().to_string()
    }

    pub fn iter_backlinks(&self, z: &Zettel) -> Vec<Rc<RefCell<Zettel>>> {
        let mut r = vec![];
        if let Some(v) = self.backlinks.get(z.zid()) {
            for idx in v.iter() {
                if let Some(z) = self.zettels.get(*idx) {
                    r.push(z.clone());
                }
            }
        }
        r
    }
}

impl IntoIterator for Kasten {
    type Item = Result<Zettel, anyhow::Error>;
    type IntoIter = SyncDiskIter;

    fn into_iter(self) -> Self::IntoIter {
        let c = (*self.config).borrow();
        SyncDiskIter {
            inner: None,
            repo_path: c.repo_path().to_string(),
        }
    }
}

pub struct SyncDiskIter {
    inner: Option<std::vec::IntoIter<DirEntry>>,
    repo_path: String,
}

impl Iterator for SyncDiskIter {
    type Item = Result<Zettel, anyhow::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_none() {
            match self.scan_markdowns() {
                Ok(mds) => {
                    self.inner = Some(mds.into_iter());
                }
                Err(_) => return None,
            }
        }

        if let Some(inner) = self.inner.as_mut() {
            if let Some(entry) = inner.next() {
                return Some(self.dir_entry_to_zettel(entry));
            }
        }
        None
    }
}

impl SyncDiskIter {
    fn scan_markdowns(&self) -> Result<Vec<DirEntry>, anyhow::Error> {
        let buf = Path::new(&self.repo_path);
        let mut dirs = vec![buf.to_path_buf()];
        let mut result = vec![];
        while let Some(cur) = dirs.pop() {
            let rd = fs::read_dir(cur)?;
            for entry in rd {
                let item = entry?;
                if item.path().is_dir() {
                    dirs.push(item.path().to_path_buf());
                } else {
                    result.push(item);
                }
            }
        }
        result.sort_by(|a, b| {
            let mut a_m = SystemTime::now();
            let mut b_m = SystemTime::now();
            if let Ok(a) = a.metadata() {
                if let Ok(am) = a.modified() {
                    a_m = am;
                }
            }
            if let Ok(b) = b.metadata() {
                if let Ok(bm) = b.modified() {
                    b_m = bm;
                }
            }
            b_m.partial_cmp(&a_m).unwrap()
        });
        Ok(result)
    }

    fn dir_entry_to_zettel(
        &self,
        entry: DirEntry,
    ) -> Result<Zettel, anyhow::Error> {
        let ze = Zettel::from_md(&self.repo_path, &entry.path())?;
        Ok(ze)
    }
}
