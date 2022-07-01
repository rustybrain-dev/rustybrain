use std::fs;
use std::fs::rename;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::str::FromStr;

use chrono::Local;
use serde::{Deserialize, Serialize};
use toml::value::Datetime;
use tree_sitter::Node;
use tree_sitter::Tree;
use tree_sitter::TreeCursor;

#[derive(Debug, Clone)]
pub struct Zettel {
    id: String,
    path: PathBuf,
    header: ZettelHeader,
    content: String,

    tree: Option<Tree>,

    #[allow(dead_code)]
    link_to: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZettelHeader {
    title: String,
    date: Option<Datetime>,

    #[allow(dead_code)]
    #[serde(skip)]
    raw: String,
}

impl ZettelHeader {
    pub fn new(title: &str) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self {
            title: title.to_string(),
            date: Some(Datetime::from_str(&today).unwrap()),
            raw: "".to_string(),
        }
    }

    pub fn from_cursor(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, anyhow::Error> {
        let raw = Self::read(cursor)?;
        let s = toml::from_str(&raw)?;
        Ok(s)
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<String, anyhow::Error> {
        let mut line_buf: String = String::new();
        let mut header: String = String::new();
        cursor.read_line(&mut line_buf)?;
        if line_buf.trim_start_matches('+').trim().is_empty() {
            loop {
                line_buf.clear();
                cursor.read_line(&mut line_buf)?;

                if line_buf.trim_start_matches('+').trim().is_empty() {
                    return Ok(header);
                }
                std::fmt::Write::write_str(&mut header, &line_buf)?;
            }
        }
        Ok(header)
    }
}

impl Zettel {
    pub fn from_md(
        repo_path: &str,
        path: &Path,
    ) -> Result<Self, anyhow::Error> {
        let mut file = File::open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let mut cursor = Cursor::new(buf);
        let header = ZettelHeader::from_cursor(&mut cursor)?;
        let mut content: String = String::new();
        cursor.read_to_string(&mut content)?;
        let id = Self::in_repo_path(path, repo_path)?;
        let tree = crate::md::parse(&content, None)?;
        let mut z = Zettel {
            id,
            path: path.to_path_buf(),
            header,
            content,
            tree,
            link_to: vec![],
        };
        z.parse_links_to();
        Ok(z)
    }

    pub fn create(
        repo_path: &str,
        path: &Path,
        title: &str,
    ) -> Result<Self, anyhow::Error> {
        Self::create_and_insert(path, title)?;
        Self::from_md(repo_path, path)
    }

    fn create_and_insert(
        path: &Path,
        title: &str,
    ) -> Result<(), anyhow::Error> {
        let mut file = File::create(path)?;
        let header = ZettelHeader::new(title);
        Self::write_header(&mut file, &header)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let tp = self.tmp();
        if tp.exists() {
            fs::remove_file(&tp)?;
        }
        let mut tmp = File::create(&tp)?;
        Self::write_header(&mut tmp, &self.header)?;
        tmp.write(self.content.as_bytes())?;
        rename(&tp, self.path())?;
        Ok(())
    }

    fn write_header(
        file: &mut File,
        header: &ZettelHeader,
    ) -> Result<(), anyhow::Error> {
        let hs = toml::to_string(&header)?;
        file.write(b"+++\n")?;
        file.write(hs.as_bytes())?;
        file.write(b"+++\n")?;
        Ok(())
    }

    fn tmp(&self) -> PathBuf {
        let dir = self.path.parent().unwrap();
        let f = self.path.file_name().unwrap().to_str().unwrap();
        dir.join(format!(".{}", f))
    }

    pub fn zid(&self) -> &str {
        &self.id
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn walk_iter(&self) -> WalkIter {
        if let Some(tree) = self.tree.as_ref() {
            return WalkIter::new(tree);
        }
        WalkIter::empty()
    }

    pub fn walk<F>(&self, on_node: F)
    where
        F: Fn(&Node),
    {
        if let Some(tree) = self.tree.as_ref() {
            let mut cursor = tree.walk();
            let mut nodes_to_deep = vec![cursor.node()];
            while let Some(node) = nodes_to_deep.pop() {
                on_node(&node);
                cursor.reset(node);
                if cursor.goto_first_child() {
                    nodes_to_deep.push(cursor.node());
                    while cursor.goto_next_sibling() {
                        nodes_to_deep.push(cursor.node());
                    }
                }
            }
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn in_repo_path(
        path: &Path,
        repo_path: &str,
    ) -> Result<String, anyhow::Error> {
        let p = path.strip_prefix(repo_path)?;
        if let Some(p) = p.to_str() {
            return Ok(format!("@/{}", p));
        }
        Err(anyhow::anyhow!("empty path"))
    }

    pub fn title(&self) -> &str {
        &self.header.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_title(&mut self, title: &str) {
        self.header.title = title.to_string();
    }

    pub fn set_content(&mut self, content: &str) -> Result<(), anyhow::Error> {
        self.tree = crate::md::parse(content, None)?;
        self.content = content.to_string();
        self.parse_links_to();
        Ok(())
    }

    fn parse_links_to(&mut self) {
        let mut link_to: Vec<String> = vec![];
        for node in self.walk_iter() {
            if node.kind() == "text" {
                if let Some(n) = node.parent() {
                    if n.kind() == "link_destination" {
                        let range = node.byte_range();
                        let link_text = &self.content.as_bytes()[range];
                        link_to.push(
                            String::from_utf8_lossy(link_text).to_string(),
                        );
                    }
                }
            }
        }
        self.link_to = link_to
    }

    pub fn link_to_iter(&self) -> Iter<'_, String> {
        self.link_to.iter()
    }
}

pub struct WalkIter<'a> {
    cursor: Option<TreeCursor<'a>>,
    nodes_to_deep: Vec<Node<'a>>,
}

impl<'a> WalkIter<'a> {
    pub fn new(tree: &'a Tree) -> Self {
        let cursor = tree.walk();
        let nodes_to_deep = vec![cursor.node()];
        Self {
            cursor: Some(cursor),
            nodes_to_deep,
        }
    }

    pub fn empty() -> Self {
        Self {
            cursor: None,
            nodes_to_deep: vec![],
        }
    }
}

impl<'a> Iterator for WalkIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.as_ref()?;
        let cursor = self.cursor.as_mut().unwrap();
        if let Some(node) = self.nodes_to_deep.pop() {
            cursor.reset(node);
            if cursor.goto_first_child() {
                self.nodes_to_deep.push(cursor.node());
                while cursor.goto_next_sibling() {
                    self.nodes_to_deep.push(cursor.node());
                }
            }
            return Some(node);
        }
        None
    }
}
