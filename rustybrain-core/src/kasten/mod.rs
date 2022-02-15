use std::fs::{self, DirEntry, ReadDir};

use crate::{
    config::Config,
    zettel::{self, Zettel, ZettelError},
};

#[derive(Debug)]
pub enum KastenError {
    ReadDirError(std::io::Error),
    ZettelIOError(std::io::Error),
    ZettelParseError(ZettelError),
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

pub struct Kasten {
    config: Config,
}

impl Kasten {
    pub fn new(config: Config) -> Self {
        Kasten { config }
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
