use std::{
    collections::HashSet,
    fs::{self, create_dir_all, DirEntry},
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::Local;
use tantivy::{
    collector::TopDocs,
    query::{QueryParser, QueryParserError},
    schema::{Field, Schema, Value, STORED, TEXT},
    Document, Index, TantivyError,
};

use crate::{
    config::Config,
    zettel::{Zettel, ZettelError},
};

#[derive(Debug)]
pub enum KastenError {
    ReadDirError(std::io::Error),
    ZettelIOError(std::io::Error),
    ZettelParseError(ZettelError),
    IndexError(tantivy::TantivyError),
    QueryError(QueryParserError),
}

impl From<std::io::Error> for KastenError {
    fn from(err: std::io::Error) -> Self {
        Self::ZettelIOError(err)
    }
}

impl From<ZettelError> for KastenError {
    fn from(err: ZettelError) -> Self {
        Self::ZettelParseError(err)
    }
}

impl From<TantivyError> for KastenError {
    fn from(e: TantivyError) -> Self {
        Self::IndexError(e)
    }
}

impl From<QueryParserError> for KastenError {
    fn from(e: QueryParserError) -> Self {
        Self::QueryError(e)
    }
}

#[derive(Clone)]
pub struct Kasten {
    config: Config,

    #[allow(dead_code)]
    schema: Schema,

    title: Field,
    body: Field,
    path: Field,
    index: Index,
}

impl Kasten {
    pub fn new(config: Config) -> Result<Self, KastenError> {
        let mut schema_builder = Schema::builder();
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let path = schema_builder.add_text_field("path", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        let kasten = Kasten {
            config,
            schema,
            index,
            title,
            body,
            path,
        };
        kasten.build_index()?;
        Ok(kasten)
    }

    fn build_index(&self) -> Result<(), KastenError> {
        {
            let mut index_writer = self.index.writer(50_000_000)?;
            index_writer.delete_all_documents()?;
            index_writer.commit()?;
        }

        for entry in self.iter() {
            let z = entry?;
            self.add_doc(z)?;
        }
        Ok(())
    }

    fn add_doc(&self, z: Zettel) -> Result<(), KastenError> {
        let mut index_writer = self.index.writer(50_000_000)?;
        let title = self.title;
        let body = self.body;
        let mut doc = Document::default();
        doc.add_text(title, z.title());
        doc.add_text(body, z.content());
        if let Some(p) = z.path().to_str() {
            doc.add_text(self.path, p);
        }
        index_writer.add_document(doc);
        index_writer.commit()?;
        Ok(())
    }

    pub fn search_title(
        &self,
        kw: &str,
    ) -> Result<HashSet<String>, KastenError> {
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
            if let Some(path) = retrieved_doc.get_first(self.path) {
                if let Value::Str(s) = path {
                    set.insert(s.to_string());
                }
            }
        }
        Ok(set)
    }

    pub fn iter(&self) -> IntoIter {
        IntoIter {
            inner: None,
            path: self.config.repo_path().to_string(),
        }
    }

    pub fn create(&self, title: &str) -> Result<Zettel, KastenError> {
        let path = self.new_path();
        if let Some(dir) = path.as_path().parent() {
            create_dir_all(dir)?;
        }
        let z = Zettel::create(&path, title)?;
        self.add_doc(z.clone())?;
        Ok(z)
    }

    pub fn save(&mut self, zettel: &Zettel) -> Result<(), KastenError> {
        zettel.save()?;
        self.build_index()?;
        Ok(())
    }

    fn new_path(&self) -> PathBuf {
        let path = self.config.repo_path();
        let gen = Local::now().format("%Y%m%d%H%M%S").to_string();
        Path::new(path)
            .join(format!("notes/{}.md", gen))
            .to_path_buf()
    }
}

impl IntoIterator for Kasten {
    type Item = Result<Zettel, KastenError>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: None,
            path: self.config.repo_path().to_string(),
        }
    }
}

pub struct IntoIter {
    inner: Option<std::vec::IntoIter<DirEntry>>,
    path: String,
}

impl Iterator for IntoIter {
    type Item = Result<Zettel, KastenError>;

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
                return Some(Self::dir_entry_to_zettel(entry));
            }
        }
        None
    }
}

impl IntoIter {
    fn scan_markdowns(&self) -> Result<Vec<DirEntry>, KastenError> {
        let buf = Path::new(&self.path);
        let mut dirs = vec![buf.to_path_buf()];
        let mut result = vec![];
        loop {
            if let Some(cur) = dirs.pop() {
                let rd = fs::read_dir(cur)?;
                for entry in rd {
                    let item = entry?;
                    if item.path().is_dir() {
                        dirs.push(item.path().to_path_buf());
                    } else {
                        result.push(item);
                    }
                }
            } else {
                break;
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

    fn dir_entry_to_zettel(entry: DirEntry) -> Result<Zettel, KastenError> {
        let ze = Zettel::from_md(&entry.path())?;
        Ok(ze)
    }
}
