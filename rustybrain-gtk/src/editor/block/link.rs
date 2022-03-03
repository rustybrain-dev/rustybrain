use gtk::{traits::TextBufferExt, TextMark};

use super::Blocking;

pub struct Link {
    left: TextMark,
    right: TextMark,
    text: Option<LinkText>,
    dest: Option<LinkDest>,
}

pub struct LinkText {
    left: TextMark,
    right: TextMark,
}

pub struct LinkDest {
    left: TextMark,
    right: TextMark,
}

impl Blocking for Link {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);

        let mut text = None;
        let mut dest = None;

        if let Some(tn) = Self::node_child_by_kind(node, "link_text") {
            text = Some(LinkText::from_node(&tn, buffer));
        }

        if let Some(dn) = Self::node_child_by_kind(node, "link_destination") {
            dest = Some(LinkDest::from_node(&dn, buffer));
        }

        Link {
            left,
            right,
            text,
            dest,
        }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn mount(&self, view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        if let Some(text) = &self.text {
            text.mount(view, buffer);
        }

        if let Some(dest) = &self.dest {
            dest.mount(view, buffer)
        }
    }

    fn umount(&self, view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        if let Some(text) = &self.text {
            text.umount(view, buffer);
        }
        if let Some(dest) = &self.dest {
            dest.umount(view, buffer);
        }
    }

    fn cursor_in(&self, view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        if let Some(text) = &self.text {
            text.cursor_in(view, buffer);
        }
        if let Some(dest) = &self.dest {
            dest.cursor_in(view, buffer);
        }
    }

    fn cursor_out(&self, view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        if let Some(text) = &self.text {
            text.cursor_out(view, buffer);
        }
        if let Some(dest) = &self.dest {
            dest.cursor_out(view, buffer);
        }
    }
}

impl Blocking for LinkText {
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

    fn mount(&self, _view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        buffer.apply_tag_by_name(
            "link",
            &self.start(buffer),
            &self.end(buffer),
        );
    }

    fn cursor_in(&self, _view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        self.show_surround(buffer);
    }

    fn cursor_out(&self, _view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        self.hide_surround(buffer);
    }
}

impl Blocking for LinkDest {
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

    fn cursor_out(&self, _view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        let mut start = self.start(buffer);
        start.backward_char();

        let mut end = self.end(buffer);
        end.forward_char();

        buffer.apply_tag_by_name("hidden", &start, &end);
    }

    fn cursor_in(&self, _view: &gtk::TextView, buffer: &gtk::TextBuffer) {
        let mut start = self.start(buffer);
        start.backward_char();

        let mut end = self.end(buffer);
        end.forward_char();

        buffer.remove_tag_by_name("hidden", &start, &end);
    }

    fn mount(&self, _view: &gtk::TextView, _buffer: &gtk::TextBuffer) {}
}
