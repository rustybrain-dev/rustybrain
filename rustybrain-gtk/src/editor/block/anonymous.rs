use super::Blocking;
use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

pub struct Anonymous {
    start: TextIter,
    end: TextIter,

    left: TextMark,
    right: TextMark,
}

impl Blocking for Anonymous {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let left = TextMark::builder().left_gravity(false).build();
        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let right = TextMark::builder().left_gravity(false).build();
        let end = buffer.iter_at_offset(node.end_byte() as i32);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
        Anonymous {
            left,
            right,
            start,
            end,
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

    fn apply_tag(&mut self, _: &TextBuffer) {}

    fn remove_tag(&self, buffer: &TextBuffer) {}
}
