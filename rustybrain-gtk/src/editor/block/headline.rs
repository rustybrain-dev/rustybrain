use super::Blocking;
use gtk::prelude::*;
use gtk::TextBuffer;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

pub struct Headline {
    left: TextMark,
    right: TextMark,

    marker: Option<Marker>,
    content: Option<Content>,
}

impl Blocking for Headline {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        let mut marker = None;
        let mut content = None;
        if let Some(child) = Self::node_child_by_kind(node, "heading_content") {
            content = Some(Content::from_node(&child, buffer));
        }
        Headline {
            left,
            right,
            marker,
            content,
        }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &TextBuffer) {
        if let Some(content) = self.content.as_ref() {
            content.apply_tag(buffer);
        }

        if let Some(marker) = self.marker.as_ref() {
            marker.apply_tag(buffer);
        }
    }

    fn remove_tag(&self, buffer: &TextBuffer) {
        if let Some(content) = self.content.as_ref() {
            content.remove_tag(buffer);
        }

        if let Some(marker) = self.marker.as_ref() {
            marker.remove_tag(buffer);
        }
    }

    fn umount(&self, buffer: &TextBuffer) {
        buffer.delete_mark(self.left());
        buffer.delete_mark(self.right());

        if let Some(content) = self.content.as_ref() {
            content.umount(buffer);
        }

        if let Some(marker) = self.marker.as_ref() {
            marker.umount(buffer);
        }
    }

    fn cursor_in(&self, buffer: &TextBuffer) {
        if let Some(content) = self.content.as_ref() {
            content.cursor_in(buffer);
        }

        if let Some(marker) = self.marker.as_ref() {
            marker.cursor_in(buffer);
        }
    }

    fn cursor_out(&self, buffer: &TextBuffer) {
        if let Some(content) = self.content.as_ref() {
            content.cursor_out(buffer);
        }

        if let Some(marker) = self.marker.as_ref() {
            marker.cursor_out(buffer);
        }
    }
}

impl Headline {
    pub fn set_number(&mut self, n: u8, node: &Node, buffer: &TextBuffer) {
        if let Some(content) = self.content.as_mut() {
            content.set_number(n)
        }

        self.marker = Some(Marker::from_node(node, buffer));
    }
}

struct Marker {
    left: TextMark,
    right: TextMark,
}

impl Blocking for Marker {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        Marker { left, right }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &TextBuffer) {
        self.hide(buffer);
    }

    fn cursor_in(&self, buffer: &TextBuffer) {
        self.show(buffer);
    }

    fn cursor_out(&self, buffer: &TextBuffer) {
        self.hide(buffer);
    }
}

impl Marker {
    fn hide(&self, buffer: &TextBuffer) {
        buffer.apply_tag_by_name(
            "hidden",
            &self.start(buffer),
            &self.end(buffer),
        );
    }

    fn show(&self, buffer: &TextBuffer) {
        buffer.remove_tag_by_name(
            "hidden",
            &self.start(buffer),
            &self.end(buffer),
        );
    }
}

struct Content {
    left: TextMark,
    right: TextMark,
    number: u8,
}

impl Blocking for Content {
    fn from_node(node: &Node, buffer: &TextBuffer) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        Content {
            left,
            right,
            number: 1,
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

impl Content {
    pub fn set_number(&mut self, n: u8) {
        self.number = n;
    }
}
