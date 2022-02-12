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
        self.fill_strikethrough();
        self.fill_hidden();
    }

    fn fill_paragraph(&mut self) {
        let fd = self.font_desc();
        let tag = TextTag::builder().name("p").font_desc(&fd).build();
        self.table.add(&tag);
    }

    fn font_desc(&self) -> FontDescription {
        let font = format!("{} {}", self.font, self.font_size);
        FontDescription::from_string(&font)
    }

    fn fill_headline(&mut self) {
        let font_sizes: Vec<i32> = vec![16, 14, 10, 8, 6, 4, 2];
        let mut hn = 1;
        for sz in font_sizes {
            let font = format!("{} {}", self.font, self.font_size + sz);
            let mut fd = FontDescription::from_string(&font);
            fd.set_style(gtk::pango::Style::Oblique);
            let name = format!("h{}", hn);
            let tag = TextTag::builder().name(&name).font_desc(&fd).build();
            self.table.add(&tag);
            hn += 1;
        }
    }

    fn fill_link(&mut self) {}

    fn fill_code(&mut self) {}

    fn fill_code_block(&mut self) {
        let fd = self.font_desc();
        let tag = TextTag::builder()
            .name("code-block")
            .paragraph_background("#c7c7c7")
            .font_desc(&fd)
            .build();
        self.table.add(&tag);
    }

    fn fill_bold(&mut self) {}

    fn fill_italic(&mut self) {}

    fn fill_strikethrough(&mut self) {}

    fn fill_hidden(&mut self) {
        let tag = TextTag::builder().name("hidden").invisible(true).build();
        self.table.add(&tag);
    }
}
