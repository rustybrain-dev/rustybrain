use gtk::prelude::*;
use gtk::TextIter;
use gtk::TextMark;

use super::Blocking;

pub struct Codeblock {
    content: Option<CodeblockContent>,

    left: TextMark,
    right: TextMark,
}

impl Blocking for Codeblock {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let mut content = None;
        let child = Self::node_child_by_kind(node, "code_fence_content");
        if let Some(cnt_node) = child {
            content = Some(CodeblockContent::from_node(&cnt_node, buffer));
        }

        let (left, right) = Self::node_endpoint(node, buffer);

        Codeblock {
            content,
            left,
            right,
        }
    }

    fn left(&self) -> &TextMark {
        &self.left
    }

    fn right(&self) -> &TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &gtk::TextBuffer) {
        if let Some(content) = &self.content {
            content.apply_tag(buffer);
            self.hide_begin_end_line(buffer);
        }
    }

    fn cursor_in(&self, buffer: &gtk::TextBuffer) {
        self.show_begin_end_line(buffer);
    }

    fn cursor_out(&self, buffer: &gtk::TextBuffer) {
        self.hide_begin_end_line(buffer);
    }

    fn start(&self, buffer: &gtk::TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.left())
    }

    fn end(&self, buffer: &gtk::TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.right())
    }

    fn remove_tag(&self, buffer: &gtk::TextBuffer) {
        if let Some(content) = &self.content {
            content.remove_tag(buffer);
            self.show_begin_end_line(buffer)
        }
    }

    fn umount(&self, buffer: &gtk::TextBuffer) {
        buffer.delete_mark(self.left());
        buffer.delete_mark(self.right());
        if let Some(content) = self.content.as_ref() {
            content.umount(buffer);
        }
    }
}

impl Codeblock {
    fn begin_line(
        &self,
        buffer: &gtk::TextBuffer,
    ) -> Option<(TextIter, TextIter)> {
        let start = buffer.iter_at_mark(self.left());
        if let Some(line) = buffer.iter_at_line(start.line() + 1) {
            let end = buffer.iter_at_offset(line.offset() - 1);
            return Some((start, end));
        }
        None
    }

    fn end_line(
        &self,
        buffer: &gtk::TextBuffer,
    ) -> Option<(TextIter, TextIter)> {
        let end = buffer.iter_at_mark(self.right());
        if let Some(line) = buffer.iter_at_line(end.line() - 1) {
            let start = buffer.iter_at_offset(line.offset());
            return Some((start, end));
        }
        None
    }

    fn hide_begin_end_line(&self, buffer: &gtk::TextBuffer) {
        if let Some((bl_start, bl_end)) = self.begin_line(buffer) {
            buffer.apply_tag_by_name("hidden", &bl_start, &bl_end);
        }
        if let Some((el_start, el_end)) = self.end_line(buffer) {
            buffer.apply_tag_by_name("hidden", &el_start, &el_end);
        }
    }

    fn show_begin_end_line(&self, buffer: &gtk::TextBuffer) {
        if let Some((bl_start, bl_end)) = self.begin_line(buffer) {
            buffer.remove_tag_by_name("hidden", &bl_start, &bl_end);
        }
        if let Some((el_start, el_end)) = self.end_line(buffer) {
            buffer.remove_tag_by_name("hidden", &el_start, &el_end);
        }
    }
}

struct CodeblockContent {
    right: TextMark,
    left: TextMark,
}

impl Blocking for CodeblockContent {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let (left, right) = Self::node_endpoint(node, buffer);
        CodeblockContent { left, right }
    }

    fn left(&self) -> &gtk::TextMark {
        &self.left
    }

    fn right(&self) -> &gtk::TextMark {
        &self.right
    }

    fn apply_tag(&self, buffer: &gtk::TextBuffer) {
        let start = self.start(buffer);
        let end = self.end(buffer);
        buffer.apply_tag_by_name("code-block", &start, &end);
    }
}
