use gtk::pango::FontDescription;
use gtk::prelude::*;
use gtk::TextTag;
use gtk::TextTagTable;

pub struct Style {
    font: String,
    font_size: i32,
    table: TextTagTable,
}

impl Style {
    pub fn new() -> Self {
        let mut style = Style {
            font: "Victor Mono".to_string(),
            font_size: 14,
            table: TextTagTable::new(),
        };
        style.fill();
        style
    }

    pub fn table(&self) -> TextTagTable {
        self.table.clone()
    }

    fn fill(&mut self) {
        self.fill_paragraph();
        self.fill_headline();
        self.fill_link();
        self.fill_code();
        self.fill_code_block();
        self.fill_bold();
        self.fill_italic();
        self.fill_strikethrough()
    }

    fn fill_paragraph(&mut self) {
        let font = format!("{} {}", self.font, self.font_size);
        let fd = FontDescription::from_string(&font);
        let tag = TextTag::builder().name("p").font_desc(&fd).build();
        self.table.add(&tag);
    }

    fn fill_headline(&mut self) {
        let font_sizes: Vec<i32> = vec![14, 12, 10, 8, 6, 4, 2];
        let mut hn = 1;
        for sz in font_sizes {
            let font = format!("{} {}", self.font, sz + self.font_size);
            let fd = FontDescription::from_string(&font);
            let name = format!("h{}", hn);
            let tag = TextTag::builder().name(&name).font_desc(&fd).build();
            self.table.add(&tag);
            hn += 1;
        }
    }

    fn fill_link(&mut self) {}

    fn fill_code(&mut self) {}

    fn fill_code_block(&mut self) {}

    fn fill_bold(&mut self) {}

    fn fill_italic(&mut self) {}

    fn fill_strikethrough(&mut self) {}
}
