use gtk::prelude::*;
use gtk::TextIter;
use gtk::TextMark;
use rustybrain_core::md::Node;

use super::Blocking;

pub struct Codeblock {
    content: Option<CodeblockContent>,

    begin_line_start: TextIter,
    begin_line_stop: TextIter,

    end_line_start: TextIter,
    end_line_stop: TextIter,

    left: TextMark,
    right: TextMark,
}

impl Blocking for Codeblock {
    fn from_node(
        node: &rustybrain_core::md::Node,
        buffer: &gtk::TextBuffer,
    ) -> Self {
        let begin_line_start =
            buffer.iter_at_line(node.start_position().row as i32);
        let begin_line_stop =
            buffer.iter_at_line(node.start_position().row as i32 + 1);

        let end_line_start =
            buffer.iter_at_line(node.end_position().row as i32);
        let end_line_stop =
            buffer.iter_at_line(node.end_position().row as i32 + 1);

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

            begin_line_stop,
            begin_line_start,

            end_line_start,
            end_line_stop,

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

    fn apply_tag(&mut self, buffer: &gtk::TextBuffer) {
        if let Some(content) = self.content.as_mut() {
            content.apply_tag(buffer);
        }
        buffer.apply_tag_by_name(
            "hidden",
            &self.begin_line_start,
            &self.begin_line_stop,
        );
        buffer.apply_tag_by_name(
            "hidden",
            &self.end_line_start,
            &self.end_line_stop,
        );
    }

    fn start(&self, buffer: &gtk::TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.left())
    }

    fn end(&self, buffer: &gtk::TextBuffer) -> gtk::TextIter {
        buffer.iter_at_mark(self.right())
    }

    fn remove_tag(&self, buffer: &gtk::TextBuffer) {
        buffer.remove_all_tags(&self.begin_line_start, &self.begin_line_stop);
        buffer.remove_all_tags(&self.end_line_start, &self.end_line_stop);

        if let Some(content) = self.content.as_ref() {
            content.remove_tag(buffer);
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

    fn apply_tag(&mut self, buffer: &gtk::TextBuffer) {
        let start = self.start(buffer);
        let end = self.end(buffer);
        buffer.apply_tag_by_name("code-block", &start, &end);
    }
}
