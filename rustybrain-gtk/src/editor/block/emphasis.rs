use gtk::{traits::TextBufferExt, TextMark};

use super::Blocking;

pub struct Emphasis {
    left: TextMark,
    right: TextMark,
}

impl Blocking for Emphasis {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        Emphasis { left, right }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &gtk::TextBuffer) {
        buffer.apply_tag_by_name(
            "italic",
            &self.start(buffer),
            &self.end(buffer),
        );
    }

    fn cursor_in(&self, buffer: &gtk::TextBuffer) {
        self.show_endpoint(buffer);
    }

    fn cursor_out(&self, buffer: &gtk::TextBuffer) {
        self.hide_endpoint(buffer);
    }
}

pub struct StrongEmphasis {
    left: TextMark,
    right: TextMark,
}

impl Blocking for StrongEmphasis {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        Self { left, right }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &gtk::TextBuffer) {
        buffer.apply_tag_by_name(
            "bold",
            &self.start(buffer),
            &self.end(buffer),
        );
    }

    fn cursor_in(&self, buffer: &gtk::TextBuffer) {
        self.show_endpoint_n(buffer, 2);
    }

    fn cursor_out(&self, buffer: &gtk::TextBuffer) {
        self.hide_endpoint_n(buffer, 2);
    }
}
