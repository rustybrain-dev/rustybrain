mod anonymous;
mod codeblock;
mod emphasis;
mod headline;
mod link;

use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use gtk::TextView;
use rustybrain_core::md::Node;

use anonymous::Anonymous;
use headline::Headline;

use self::codeblock::Codeblock;
use self::emphasis::Emphasis;
use self::emphasis::StrongEmphasis;
use self::link::Link;

pub trait Blocking {
    fn node_endpoint(node: &Node, buffer: &TextBuffer) -> (TextMark, TextMark) {
        let left = TextMark::builder().left_gravity(false).build();
        let right = TextMark::builder().left_gravity(false).build();
        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let end = buffer.iter_at_offset(node.end_byte() as i32);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
        (left, right)
    }

    fn node_child_by_kind<'a>(
        node: &'a Node,
        name: &'a str,
    ) -> Option<Node<'a>> {
        (0_usize..node.child_count())
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

    fn mount(&self, view: &TextView, buffer: &TextBuffer);

    fn start(&self, buffer: &TextBuffer) -> TextIter {
        buffer.iter_at_mark(self.left())
    }

    fn end(&self, buffer: &TextBuffer) -> TextIter {
        buffer.iter_at_mark(self.right())
    }

    fn umount(&self, _view: &TextView, buffer: &TextBuffer) {
        let start = self.start(buffer);
        let end = self.end(buffer);
        buffer.remove_all_tags(&start, &end);
        buffer.delete_mark(self.left());
        buffer.delete_mark(self.right());
    }

    fn cursor_in(&self, _view: &TextView, _buffer: &TextBuffer) {}

    fn cursor_out(&self, _view: &TextView, _buffer: &TextBuffer) {}

    fn hide_surround(&self, buffer: &gtk::TextBuffer) {
        let ((b_start, b_end), (e_start, e_end)) = self.surround(buffer, 1);
        buffer.apply_tag_by_name("hidden", &b_start, &b_end);
        buffer.apply_tag_by_name("hidden", &e_start, &e_end);
    }

    fn show_surround(&self, buffer: &gtk::TextBuffer) {
        let ((b_start, b_end), (e_start, e_end)) = self.surround(buffer, 1);
        buffer.remove_tag_by_name("hidden", &b_start, &b_end);
        buffer.remove_tag_by_name("hidden", &e_start, &e_end);
    }

    fn surround(
        &self,
        buffer: &gtk::TextBuffer,
        n: i32,
    ) -> ((TextIter, TextIter), (TextIter, TextIter)) {
        let b_end = self.start(buffer);
        let mut b_start = b_end;
        (0..n).for_each(|_| {
            b_start.backward_char();
        });

        let e_start = self.end(buffer);
        let mut e_end = e_start;
        (0..n).for_each(|_| {
            e_end.forward_char();
        });
        ((b_end, b_start), (e_start, e_end))
    }

    fn hide_endpoint(&self, buffer: &gtk::TextBuffer) {
        let ((b_start, b_end), (e_start, e_end)) = self.endpoint(buffer, 1);
        buffer.apply_tag_by_name("hidden", &b_start, &b_end);
        buffer.apply_tag_by_name("hidden", &e_start, &e_end);
    }

    fn show_endpoint(&self, buffer: &gtk::TextBuffer) {
        let ((b_start, b_end), (e_start, e_end)) = self.endpoint(buffer, 1);
        buffer.remove_tag_by_name("hidden", &b_start, &b_end);
        buffer.remove_tag_by_name("hidden", &e_start, &e_end);
    }

    fn hide_endpoint_n(&self, buffer: &gtk::TextBuffer, n: i32) {
        let ((b_start, b_end), (e_start, e_end)) = self.endpoint(buffer, n);
        buffer.apply_tag_by_name("hidden", &b_start, &b_end);
        buffer.apply_tag_by_name("hidden", &e_start, &e_end);
    }

    fn show_endpoint_n(&self, buffer: &gtk::TextBuffer, n: i32) {
        let ((b_start, b_end), (e_start, e_end)) = self.endpoint(buffer, n);
        buffer.remove_tag_by_name("hidden", &b_start, &b_end);
        buffer.remove_tag_by_name("hidden", &e_start, &e_end);
    }

    fn endpoint(
        &self,
        buffer: &gtk::TextBuffer,
        n: i32,
    ) -> ((TextIter, TextIter), (TextIter, TextIter)) {
        let b_start = self.start(buffer);
        let mut b_end = b_start;

        (0..n).for_each(|_| {
            b_end.forward_char();
        });

        let e_end = self.end(buffer);
        let mut e_start = e_end;
        for _ in 0..n {
            e_start.backward_char();
        }
        ((b_end, b_start), (e_start, e_end))
    }
}

pub enum Block {
    Headline(Headline),
    Codeblock(Codeblock),
    Link(Link),
    Emphasis(Emphasis),
    StrongEmphasis(StrongEmphasis),
    Anonymous(Anonymous),
}

impl Blocking for Block {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        for n in 1..8 {
            if node.kind() == format!("atx_h{}_marker", n) {
                if let Some(p) = node.parent().as_ref() {
                    let mut headline = Headline::from_node(p, buffer);
                    headline.set_number(n, node, buffer);
                    return Self::Headline(headline);
                }
            }
        }
        if node.kind() == "fenced_code_block" {
            return Self::Codeblock(Codeblock::from_node(node, buffer));
        }
        if node.kind() == "link" {
            return Self::Link(Link::from_node(node, buffer));
        }
        if node.kind() == "emphasis" {
            return Self::Emphasis(Emphasis::from_node(node, buffer));
        }
        if node.kind() == "strong_emphasis" {
            return Self::StrongEmphasis(StrongEmphasis::from_node(
                node, buffer,
            ));
        }

        Self::Anonymous(Anonymous::from_node(node, buffer))
    }

    fn start(&self, buffer: &TextBuffer) -> TextIter {
        match self {
            Block::Headline(h) => h.start(buffer),
            Block::Anonymous(a) => a.start(buffer),
            Block::Codeblock(b) => b.start(buffer),
            Block::Link(l) => l.start(buffer),
            Block::Emphasis(e) => e.start(buffer),
            Block::StrongEmphasis(s) => s.start(buffer),
        }
    }

    fn end(&self, buffer: &TextBuffer) -> TextIter {
        match self {
            Block::Headline(h) => h.end(buffer),
            Block::Anonymous(a) => a.end(buffer),
            Block::Codeblock(b) => b.end(buffer),
            Block::Link(l) => l.end(buffer),
            Block::Emphasis(e) => e.end(buffer),
            Block::StrongEmphasis(s) => s.end(buffer),
        }
    }

    fn left(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.left(),
            Block::Anonymous(a) => a.left(),
            Block::Codeblock(b) => b.left(),
            Block::Link(l) => l.left(),
            Block::Emphasis(e) => e.left(),
            Block::StrongEmphasis(s) => s.left(),
        }
    }

    fn right(&self) -> &TextMark {
        match self {
            Block::Headline(h) => h.right(),
            Block::Anonymous(a) => a.right(),
            Block::Codeblock(b) => b.right(),
            Block::Link(l) => l.right(),
            Block::Emphasis(e) => e.right(),
            Block::StrongEmphasis(s) => s.right(),
        }
    }

    fn mount(&self, view: &TextView, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.mount(view, buffer),
            Block::Anonymous(a) => a.mount(view, buffer),
            Block::Codeblock(b) => b.mount(view, buffer),
            Block::Link(l) => l.mount(view, buffer),
            Block::Emphasis(e) => e.mount(view, buffer),
            Block::StrongEmphasis(s) => s.mount(view, buffer),
        }
    }

    fn umount(&self, view: &TextView, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.umount(view, buffer),
            Block::Anonymous(a) => a.umount(view, buffer),
            Block::Codeblock(b) => b.umount(view, buffer),
            Block::Link(l) => l.umount(view, buffer),
            Block::Emphasis(e) => e.umount(view, buffer),
            Block::StrongEmphasis(s) => s.umount(view, buffer),
        }
    }

    fn cursor_in(&self, view: &TextView, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.cursor_in(view, buffer),
            Block::Codeblock(c) => c.cursor_in(view, buffer),
            Block::Anonymous(a) => a.cursor_in(view, buffer),
            Block::Link(l) => l.cursor_in(view, buffer),
            Block::Emphasis(e) => e.cursor_in(view, buffer),
            Block::StrongEmphasis(s) => s.cursor_in(view, buffer),
        }
    }

    fn cursor_out(&self, view: &TextView, buffer: &TextBuffer) {
        match self {
            Block::Headline(h) => h.cursor_out(view, buffer),
            Block::Codeblock(h) => h.cursor_out(view, buffer),
            Block::Anonymous(h) => h.cursor_out(view, buffer),
            Block::Link(l) => l.cursor_out(view, buffer),
            Block::Emphasis(e) => e.cursor_out(view, buffer),
            Block::StrongEmphasis(s) => s.cursor_out(view, buffer),
        }
    }
}

impl Block {
    pub fn is_anonymous(&self) -> bool {
        matches!(self, Block::Anonymous(_))
    }
}
