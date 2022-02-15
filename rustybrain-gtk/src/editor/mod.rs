mod block;
mod style;

use gtk::{prelude::*, Adjustment, ScrolledWindow};
use relm::connect;
use relm::{Update, Widget};
use relm_derive::Msg;
use rustybrain_core::config::Config;
use rustybrain_core::md::TreeCursor;
use rustybrain_core::md::{Node, Tree};

use self::block::Blocking;

pub struct Model {
    tree: Option<Tree>,
}

#[derive(Msg)]
pub enum Msg {
    Changed,
    Cursor,
}

pub struct Editor {
    model: Model,
    window: gtk::ScrolledWindow,
    buffer: gtk::TextBuffer,
    blocks: Vec<block::Block>,
}

const CONTENT: &'static str = r#"# tree-sitter-markdown

Markdown ([CommonMark Spec v0.29-gfm](https://github.github.com/gfm/)) grammar
for [tree-sitter](https://github.com/tree-sitter/tree-sitter)

_Note: This grammar is based on the assumption that
**[link label matchings](https://github.github.com/gfm/#matches) will never fail**
since reference links can come before their reference definitions,
which causes it hard to do incrementally parsing without this assumption._

[Changelog](https://github.com/ikatyang/tree-sitter-markdown/blob/master/CHANGELOG.md)

## Install

```sh
npm install tree-sitter-markdown tree-sitter
```

## Usage

```js
const Parser = require("tree-sitter");
const Markdown = require("tree-sitter-markdown");

const parser = new Parser();
parser.setLanguage(Markdown);

const sourceCode = `
# foo
-     bar
  baz
`;

const tree = parser.parse(sourceCode);
console.log(tree.rootNode.toString());
// (document
//   (atx_heading
//     (atx_heading_marker)
//     (heading_content))
//   (tight_list
//     (list_item
//       (list_marker)
//       (indented_code_block)
//       (paragraph))))
```

## License

MIT © [Ika](https://github.com/ikatyang)
"#;

impl Editor {
    fn on_buffer_changed(&mut self) {
        while let Some(blk) = self.blocks.pop() {
            blk.remove_tag(&self.buffer);
            blk.umount(&self.buffer);
        }

        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();

        self.buffer.remove_all_tags(&start, &end);
        self.buffer.apply_tag_by_name("p", &start, &end);

        let raw_text = self.buffer.text(&start, &end, true);
        if raw_text.is_none() {
            return;
        }
        let text = raw_text.unwrap();

        if let Ok(tree) = rustybrain_core::md::parse(text.as_str(), None) {
            self.model.tree = tree;
        } else {
            self.model.tree = None;
        }

        if let Some(tree) = self.model.tree.clone() {
            self.walk(tree.walk());
        }
    }

    fn walk(&mut self, mut cursor: TreeCursor) {
        let mut nodes_to_deep = vec![cursor.node()];
        while let Some(node) = nodes_to_deep.pop() {
            self.on_node(&node);
            cursor.reset(node);
            if cursor.goto_first_child() {
                nodes_to_deep.push(cursor.node());
                while cursor.goto_next_sibling() {
                    nodes_to_deep.push(cursor.node());
                }
            }
        }
    }

    fn on_node(&mut self, node: &Node) {
        let blk = block::Block::from_node(node, &mut self.buffer);
        blk.apply_tag(&self.buffer);
        self.blocks.push(blk);
    }

    fn on_cursor_notify(&mut self) {
        let offset = self.buffer.cursor_position();

        for blk in &self.blocks {
            if blk.is_anonymous() {
                continue;
            }

            if blk.start(&self.buffer).offset() <= offset
                && blk.end(&self.buffer).offset() > offset
            {
                blk.cursor_in(&self.buffer)
            } else {
                blk.cursor_out(&self.buffer)
            }
        }
    }
}

impl Update for Editor {
    type Model = Model;

    type ModelParam = Config;

    type Msg = Msg;

    fn model(
        _relm: &relm::Relm<Self>,
        _paramm: Self::ModelParam,
    ) -> Self::Model {
        Model { tree: None }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Changed => self.on_buffer_changed(),
            Msg::Cursor => self.on_cursor_notify(),
        }
    }
}

impl Widget for Editor {
    type Root = gtk::ScrolledWindow;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let tag_table = style::Style::new().table();
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let buffer = gtk::TextBuffer::new(Some(&tag_table));
        buffer.set_text(CONTENT);
        connect!(
            relm,
            buffer,
            connect_changed(_),
            return (Some(Msg::Changed), ())
        );
        connect!(
            relm,
            buffer,
            connect_cursor_position_notify(_),
            return (Some(Msg::Cursor), ())
        );
        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .width_request(800)
            .height_request(600)
            .has_tooltip(true)
            .margin(10)
            .wrap_mode(gtk::WrapMode::Char)
            .build();
        view.set_size_request(800, 600);
        box_.add(&view);

        let hadj = Adjustment::builder().build();
        let window =
            ScrolledWindow::new::<Adjustment, Adjustment>(Some(&hadj), None);
        window.set_child(Some(&box_));
        window.show_all();

        Editor {
            model,
            window,
            buffer,
            blocks: vec![],
        }
    }

    fn init_view(&mut self) {
        self.on_buffer_changed();
    }

    fn on_add<W: IsA<gtk::Widget> + IsA<relm::Object>>(&self, _parent: W) {}

    fn parent_id() -> Option<&'static str> {
        None
    }

    fn run(model_param: Self::ModelParam) -> Result<(), gtk::glib::BoolError>
    where
        Self: 'static,
    {
        relm::run::<Self>(model_param)
    }
}
