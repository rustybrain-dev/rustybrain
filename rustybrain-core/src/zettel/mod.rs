use std::fmt::Write;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

#[derive(Debug)]
pub enum ZettelError {
    IOError(io::Error),
    ParseHeaderError(toml::de::Error),
    BuildHeaderError(std::fmt::Error),
}

impl From<io::Error> for ZettelError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<toml::de::Error> for ZettelError {
    fn from(e: toml::de::Error) -> Self {
        Self::ParseHeaderError(e)
    }
}

impl From<std::fmt::Error> for ZettelError {
    fn from(err: std::fmt::Error) -> Self {
        ZettelError::BuildHeaderError(err)
    }
}

#[derive(Debug, Clone)]
pub struct Zettel {
    path: PathBuf,
    header: ZettelHeader,
    content: String,
    link_to: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZettelHeader {
    title: String,

    #[serde(skip)]
    raw: String,
}

impl ZettelHeader {
    pub fn from_cursor(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, ZettelError> {
        let raw = Self::read(cursor)?;
        let s = toml::from_str(&raw)?;
        Ok(s)
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<String, ZettelError> {
        let mut line_buf: String = String::new();
        let mut header: String = String::new();
        cursor.read_line(&mut line_buf)?;
        if line_buf.trim_start_matches("+").trim().len() == 0 {
            loop {
                line_buf.clear();
                cursor.read_line(&mut &mut line_buf)?;

                if line_buf.trim_start_matches("+").trim().len() == 0 {
                    return Ok(header);
                }
                header.write_str(&line_buf)?;
            }
        }
        Ok(header)
    }
}

impl Zettel {
    pub fn from_md(path: &Path) -> Result<Self, ZettelError> {
        let mut file = File::open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let mut cursor = Cursor::new(buf);
        let header = ZettelHeader::from_cursor(&mut cursor)?;
        let mut content: String = String::new();
        cursor.read_to_string(&mut content)?;
        Ok(Zettel {
            path: path.to_path_buf(),
            header,
            content,
            link_to: vec![],
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn title(&self) -> &str {
        &self.header.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_title(&mut self, _title: &str) {}

    pub fn set_content(&mut self, _content: &str) {}

    pub fn save() {}
}
