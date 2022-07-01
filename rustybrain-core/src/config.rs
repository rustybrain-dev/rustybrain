use std::env::var;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    repo: Repo,
    shortcut: Shortcut,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Shortcut {
    find: String,
    insert: String,
    quit: String,
}

impl Config {
    pub fn repo_path(&self) -> &str {
        &self.repo.path
    }

    pub fn shortcut(&self) -> &Shortcut {
        &self.shortcut
    }
}

impl std::str::FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = toml::from_str(s)?;
        Ok(config)
    }
}

impl Shortcut {
    pub fn find(&self) -> &str {
        &self.find
    }

    pub fn insert(&self) -> &str {
        &self.insert
    }

    pub fn quit(&self) -> &str {
        &self.quit
    }
}

#[derive(Default)]
pub struct ConfigLoader {
    #[allow(dead_code)]
    home: PathBuf,
    dir: PathBuf,
    path: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let raw_home = var("HOME").unwrap();
        let home = Path::new(&raw_home).to_path_buf();
        let dir = home.join(".rustybrain");
        let path = dir.join("config.toml");
        ConfigLoader { home, dir, path }
    }

    pub fn load(&self) -> Result<Config, anyhow::Error> {
        self.create_dir()?;
        self.attempt_set_default()?;
        self.load_config()
    }

    fn create_dir(&self) -> Result<(), io::Error> {
        if Self::is_exists(&self.dir) {
            return Ok(());
        }
        fs::create_dir(&self.dir)?;
        Ok(())
    }

    fn is_exists(path: &Path) -> bool {
        fs::metadata(path).is_ok()
    }

    fn attempt_set_default(&self) -> Result<(), io::Error> {
        match File::open(&self.path) {
            Ok(_) => Ok(()),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => self.create_default(),
                _ => Err(err),
            },
        }
    }

    fn create_default(&self) -> Result<(), io::Error> {
        let mut f = File::create(&self.path)?;
        f.write_all(DEFAULT_CONFIG_CONTENT.as_bytes())?;
        Ok(())
    }

    fn load_config(&self) -> Result<Config, anyhow::Error> {
        let mut f = File::open(&self.path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        let s = from_utf8(&buf)?;
        s.parse()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Repo {
    path: String,
}

const DEFAULT_CONFIG_CONTENT: &str = r###"
[repo]
path = "RustyBrain"

[shortcut]
find = "<Control><Shift>f"
insert = "<Control>i"
quit = "<Meta>q"

"###;

#[cfg(test)]
mod tests {
    use super::ConfigLoader;

    #[test]
    fn test_default_config_loader() {
        let loader = ConfigLoader::new();
        loader.load().unwrap();
    }
}
