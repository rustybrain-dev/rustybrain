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
        let (left, right) = Self::node_endpoint(node, buffer);
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
