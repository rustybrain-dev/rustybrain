use tree_sitter::Tree;

pub struct Zettel {
    path: String,
    title: String,
    link_to: Vec<String>,
    content: String,
}

impl Zettel {
    pub fn from_md(path: &str) -> Result<Self, String> {
        todo!()
    }

    pub fn path(&self) -> &str {
        todo!()
    }

    pub fn title(&self) -> &str {
        todo!()
    }

    pub fn content(&self) -> &str {
        todo!()
    }

    pub fn contexts(&self) -> &str {
        todo!()
    }

    pub fn tree(&self) -> Tree {
        todo!()
    }

    pub fn set_title(&mut self, title: &str) {}

    pub fn set_content(&mut self, content: &str) {}

    pub fn save() {}
}
