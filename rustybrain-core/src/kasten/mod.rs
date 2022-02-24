use std::fs::{self, DirEntry, ReadDir};

use tantivy::{
    schema::{Field, Schema, STORED, TEXT},
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
        let mut index_writer = self.index.writer(50_000_000)?;
        let title = self.title;
        let body = self.body;
        for entry in self.clone() {
            let z = entry?;
            let mut doc = Document::default();
            doc.add_text(title, z.title());
            doc.add_text(body, z.content());
            if let Some(p) = z.path().to_str() {
                doc.add_text(self.path, p);
            }
            index_writer.add_document(doc);
        }
        index_writer.commit()?;
        Ok(())
    }
}

impl IntoIterator for Kasten {
    type Item = Result<Zettel, KastenError>;
    type IntoIter = KastenIter;

    fn into_iter(self) -> Self::IntoIter {
        KastenIter {
            inner: None,
            path: self.config.repo_path().to_string(),
        }
    }
}

pub struct KastenIter {
    inner: Option<ReadDir>,
    path: String,
}

impl Iterator for KastenIter {
    type Item = Result<Zettel, KastenError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_none() {
            let inner = fs::read_dir(&self.path);
            match inner {
                Ok(inner) => self.inner = Some(inner),
                Err(_) => {
                    return None;
                }
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

impl KastenIter {
    fn dir_entry_to_zettel(
        entry: Result<DirEntry, std::io::Error>,
    ) -> Result<Zettel, KastenError> {
        let item = entry?;
        let ze = Zettel::from_md(&item.path())?;
        Ok(ze)
    }
}
