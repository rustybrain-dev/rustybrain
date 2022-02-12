use super::Blocking;
use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextMark;
use rustybrain_core::md::Node;

pub struct Headline {
    left: TextMark,
    right: TextMark,

    number: u8,
}

impl Blocking for Headline {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let left = TextMark::builder().left_gravity(false).build();
        let right = TextMark::builder().left_gravity(false).build();

        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let end = buffer.iter_at_offset(node.end_byte() as i32 + 1);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
        Headline {
            left,
            right,
            number: 6,
        }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &TextBuffer) {
        let start = self.start(buffer);
        let end = self.end(buffer);
        let tag = format!("h{}", self.number);
        buffer.apply_tag_by_name(&tag, &start, &end);
    }
}

impl Headline {
    pub fn set_number(&mut self, n: u8) {
        self.number = n;
    }
}
