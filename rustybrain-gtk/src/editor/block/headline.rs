use super::Blocking;
use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

pub struct Headline {
    start: TextIter,
    end: TextIter,

    left: TextMark,
    right: TextMark,

    number: u8,
}

impl Blocking for Headline {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let left = TextMark::builder().left_gravity(false).build();
        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let right = TextMark::builder().left_gravity(false).build();
        let end = buffer.iter_at_offset(node.end_byte() as i32 + 1);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
        Headline {
            left,
            right,
            start,
            end,
            number: 6,
        }
    }

    fn start(&self) -> &TextIter {
        &self.start
    }

    fn end(&self) -> &TextIter {
        &self.end
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn umount(&self, buffer: &TextBuffer) {}

    fn apply_tag(&mut self, buffer: &TextBuffer) {
        let start = buffer.iter_at_mark(self.left());
        let end = buffer.iter_at_mark(self.right());
        let tag = format!("h{}", self.number);
        buffer.apply_tag_by_name(&tag, &start, &end);

        self.start = start;
        self.end = end;
    }

    fn remove_tag(&self, buffer: &TextBuffer) {
        buffer.remove_all_tags(self.start(), self.end());
    }
}

impl Headline {
    pub fn set_number(&mut self, n: u8) {
        self.number = n;
    }
}
