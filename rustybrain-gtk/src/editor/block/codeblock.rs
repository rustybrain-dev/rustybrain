use gtk::prelude::*;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

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
        let child: Option<Node> = (0 as usize..node.child_count())
            .filter_map(|i| {
                if let Some(n) = node.child(i) {
                    if n.kind() == "code_fence_content" {
                        return Some(n);
                    }
                }
                None
            })
            .last();
        if let Some(cnt_node) = child {
            content = Some(CodeblockContent::from_node(&cnt_node, buffer));
        }

        let left = TextMark::builder().left_gravity(false).build();
        let right = TextMark::builder().left_gravity(false).build();

        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let end = buffer.iter_at_offset(node.end_byte() as i32 + 1);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);

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
    fn begin_line(&self, buffer: &gtk::TextBuffer) -> (TextIter, TextIter) {
        let start = buffer.iter_at_mark(self.left());
        let end = buffer
            .iter_at_offset(buffer.iter_at_line(start.line() + 1).offset() - 1);
        (start, end)
    }

    fn end_line(&self, buffer: &gtk::TextBuffer) -> (TextIter, TextIter) {
        let end = buffer.iter_at_mark(self.right());
        let start =
            buffer.iter_at_offset(buffer.iter_at_line(end.line() - 1).offset());
        (start, end)
    }

    fn hide_begin_end_line(&self, buffer: &gtk::TextBuffer) {
        let (bl_start, bl_end) = self.begin_line(buffer);
        let (el_start, el_end) = self.end_line(buffer);

        buffer.apply_tag_by_name("hidden", &bl_start, &bl_end);
        buffer.apply_tag_by_name("hidden", &el_start, &el_end);
    }

    fn show_begin_end_line(&self, buffer: &gtk::TextBuffer) {
        let (bl_start, bl_end) = self.begin_line(buffer);
        let (el_start, el_end) = self.end_line(buffer);

        buffer.remove_tag_by_name("hidden", &bl_start, &bl_end);
        buffer.remove_tag_by_name("hidden", &el_start, &el_end);
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
        let left = TextMark::builder().left_gravity(false).build();
        let right = TextMark::builder().left_gravity(false).build();

        let start = buffer.iter_at_offset(node.start_byte() as i32);
        let end = buffer.iter_at_offset(node.end_byte() as i32 + 1);
        buffer.add_mark(&left, &start);
        buffer.add_mark(&right, &end);
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

impl CodeblockContent {
    pub fn length(&self, buffer: &gtk::TextBuffer) -> i32 {
        return self.end(buffer).offset() - self.start(buffer).offset();
    }
}
