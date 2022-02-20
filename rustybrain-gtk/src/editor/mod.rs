mod block;
mod style;

use gtk::{prelude::*, ScrolledWindow};
use relm4::{send, AppUpdate, ComponentUpdate, Widgets};
use rustybrain_core::config::Config;
use rustybrain_core::md::TreeCursor;
use rustybrain_core::md::{Node, Tree};

use self::block::Blocking;

pub struct Model {
    tree: Option<Tree>,
    config: Config,
    buffer: gtk::TextBuffer,

    #[allow(dead_code)]
    table: gtk::TextTagTable,
    blocks: Vec<block::Block>,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Editor;

    type Components = ();
}

impl AppUpdate for Model {
    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        println!("update");
        match msg {
            Msg::Changed => self.on_buffer_changed(),
            Msg::Cursor => self.on_cursor_notify(),
        };
        true
    }
}

pub enum Msg {
    Changed,
    Cursor,
}

pub struct Editor {
    window: gtk::ScrolledWindow,
}

impl Model {
    fn on_buffer_changed(&mut self) {
        while let Some(blk) = self.blocks.pop() {
            blk.remove_tag(&self.buffer);
            blk.umount(&self.buffer);
        }

        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();

        self.buffer.remove_all_tags(&start, &end);
        self.buffer.apply_tag_by_name("p", &start, &end);

        let text = self.buffer.text(&start, &end, true);

        if let Ok(tree) = rustybrain_core::md::parse(text.as_str(), None) {
            self.tree = tree;
        } else {
            self.tree = None;
        }

        if let Some(tree) = self.tree.clone() {
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

impl ComponentUpdate<super::AppModel> for Model {
    fn init_model(parent_model: &super::AppModel) -> Self {
        let table = style::Style::new().table();
        let buffer = gtk::TextBuffer::builder()
            .enable_undo(true)
            .tag_table(&table)
            .build();

        buffer.set_text(CONTENT);

        let mut model = Model {
            tree: None,
            config: parent_model.config.clone(),
            blocks: vec![],
            buffer,
            table,
        };
        model.on_buffer_changed();
        model
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        _parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Changed => self.on_buffer_changed(),
            Msg::Cursor => self.on_cursor_notify(),
        }
    }
}

impl Widgets<Model, super::AppModel> for Editor {
    type Root = gtk::ScrolledWindow;

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn init_view(
        model: &Model,
        _components: &(),
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let s = sender.clone();
        model
            .buffer
            .connect_changed(move |_| send!(s, Msg::Changed));
        model.buffer.connect_cursor_position_notify(move |_| {
            send!(sender.clone(), Msg::Cursor)
        });
        let view = gtk::TextView::builder()
            .buffer(&model.buffer)
            .width_request(800)
            .height_request(600)
            .vexpand(true)
            .hexpand(true)
            .has_tooltip(true)
            .wrap_mode(gtk::WrapMode::Char)
            .build();
        box_.append(&view);
        box_.set_hexpand(true);
        box_.set_vexpand(true);

        let window = ScrolledWindow::new();
        window.set_child(Some(&box_));
        window.set_hexpand(true);
        window.set_vexpand(true);

        Editor { window }
    }

    fn view(&mut self, model: &Model, sender: relm4::Sender<Msg>) {}
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
