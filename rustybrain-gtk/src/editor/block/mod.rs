mod anonymous;
mod codeblock;
mod headline;
mod link;

use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

use anonymous::Anonymous;
use headline::Headline;

use self::codeblock::Codeblock;

pub trait Blocking {
    fn node_endpoint(node: &Node, buffer: &TextBuffer) -> (TextMark, TextMark) {
        let left = TextMark::builder().left_gravity(false).build();
        let right = TextMark::builder().left_gravity(false).build();
        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let end = buffer.iter_at_offset(node.end_byte() as i32 + 1);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
        (left, right)
    }

    fn node_child_by_kind<'a>(
        node: &'a Node,
        name: &'a str,
    ) -> Option<Node<'a>> {
        (0 as usize..node.child_count())
            .filter_map(|i| {
                if let Some(n) = node.child(i) {
                    if n.kind() == name {
                        return Some(n);
                    }
                }
                None
            })
            .last()
    }

    fn from_node(node: &Node, buffer: &TextBuffer) -> Self;

    fn left(&self) -> &TextMark;
    fn right(&self) -> &TextMark;

    fn apply_tag(&self, buffer: &TextBuffer);

    fn start(&self, buffer: &TextBuffer) -> TextIter {
        buffer.iter_at_mark(self.left())
    }

    fn end(&self, buffer: &TextBuffer) -> TextIter {
        buffer.iter_at_mark(self.right())
    }

    fn remove_tag(&self, buffer: &TextBuffer) {
        let start = self.start(buffer);
        let end = self.end(buffer);
        buffer.remove_all_tags(&start, &end);
    }

    fn umount(&self, buffer: &TextBuffer) {
        buffer.delete_mark(self.left());
        buffer.delete_mark(self.right());
    }

    fn cursor_in(&self, buffer: &TextBuffer) {}

    fn cursor_out(&self, buffer: &TextBuffer) {}
}

pub enum Block {
    Headline(Headline),
    Codeblock(Codeblock),
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
        if node.kind() == "fenced_code_block" {
            return Self::Codeblock(Codeblock::from_node(node, buffer));
        }
        Self::Anonymous(Anonymous::from_node(node, buffer))
    }

    fn start(&self, buffer: &TextBuffer) -> TextIter {
        match self {
            Block::Headline(h) => h.start(buffer),
            Block::Anonymous(a) => a.start(buffer),
            Block::Codeblock(b) => b.start(buffer),
        }
    }

    fn end(&self, buffer: &TextBuffer) -> TextIter {
        match self {
            Block::Headline(h) => h.end(buffer),
            Block::Anonymous(a) => a.end(buffer),
            Block::Codeblock(b) => b.end(buffer),
        }
    }

    fn left(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.left(),
            Block::Anonymous(a) => a.left(),
            Block::Codeblock(b) => b.left(),
        }
    }

    fn right(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.right(),
            Block::Anonymous(a) => a.right(),
            Block::Codeblock(b) => b.right(),
        }
    }

    fn apply_tag(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.apply_tag(buffer),
            Block::Anonymous(a) => a.apply_tag(buffer),
            Block::Codeblock(b) => b.apply_tag(buffer),
        }
    }

    fn umount(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.umount(buffer),
            Block::Anonymous(a) => a.umount(buffer),
            Block::Codeblock(b) => b.umount(buffer),
        }
    }

    fn remove_tag(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.remove_tag(buffer),
            Block::Anonymous(a) => a.remove_tag(buffer),
            Block::Codeblock(b) => b.remove_tag(buffer),
        }
    }

    fn cursor_in(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.cursor_in(buffer),
            Block::Codeblock(c) => c.cursor_in(buffer),
            Block::Anonymous(a) => a.cursor_in(buffer),
        }
    }

    fn cursor_out(&self, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.cursor_out(buffer),
            Block::Codeblock(h) => h.cursor_out(buffer),
            Block::Anonymous(h) => h.cursor_out(buffer),
        }
    }
}

impl Block {
    pub fn is_anonymous(&self) -> bool {
        if let Block::Anonymous(_) = self {
            true
        } else {
            false
        }
    }
}
