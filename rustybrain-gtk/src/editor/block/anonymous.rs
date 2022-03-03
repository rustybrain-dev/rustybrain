use super::Blocking;
use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextMark;
use gtk::TextView;
use rustybrain_core::md::Node;

pub struct Anonymous {
    left: TextMark,
    right: TextMark,
}

impl Blocking for Anonymous {
    fn from_node(_node: &Node, _buffer: &TextBuffer) -> Self {
        Anonymous {
            left: TextMark::new(None, true),
            right: TextMark::new(None, true),
        }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn mount(&self, _: &TextView, _: &TextBuffer) {}

    fn start(&self, buffer: &TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.left())
    }

    fn end(&self, buffer: &TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.right())
    }

    fn umount(&self, _v: &TextView, _buffer: &TextBuffer) {}
}
