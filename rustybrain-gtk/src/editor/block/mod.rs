pub mod anonymous;
pub mod headline;

use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

pub use anonymous::Anonymous;
pub use headline::Headline;

pub trait Blocking {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self;

    fn start(&self) -> &TextIter;
    fn end(&self) -> &TextIter;

    fn left(&self) -> &TextMark;
    fn right(&self) -> &TextMark;

    fn remove_tag(&self, buffer: &TextBuffer);
    fn apply_tag(&mut self, buffer: &TextBuffer);

    fn umount(&self, buffer: &TextBuffer);
}

pub enum Block {
    Headline(Headline),
    Anonymous(Anonymous),
}

impl Blocking for Block {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        for n in 1..8 {
            if node.kind() == format!("atx_h{}_marker", n) {
                if let Some(p) = node.parent().as_ref() {
                    let mut headline = Headline::from_node(p, buffer);
                    headline.set_number(n);
                    return Self::Headline(headline);
                }
            }
        }
        Self::Anonymous(Anonymous::from_node(node, buffer))
    }

    fn start(&self) -> &TextIter {
        match self {
            Block::Headline(h) => h.start(),
            Block::Anonymous(a) => a.start(),
        }
    }

    fn end(&self) -> &TextIter {
        match self {
            Block::Headline(h) => h.end(),
            Block::Anonymous(a) => a.end(),
        }
    }

    fn left(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.left(),
            Block::Anonymous(a) => a.left(),
        }
    }

    fn right(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.right(),
            Block::Anonymous(a) => a.right(),
        }
    }

    fn apply_tag(&mut self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.apply_tag(buffer),
            Block::Anonymous(a) => a.apply_tag(buffer),
        }
    }

    fn umount(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.umount(buffer),
            Block::Anonymous(a) => a.umount(buffer),
        }
    }

    fn remove_tag(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.remove_tag(buffer),
            Block::Anonymous(a) => a.remove_tag(buffer),
        }
    }
}
