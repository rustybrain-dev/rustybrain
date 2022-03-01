mod block;
mod style;

use gtk::{prelude::*, ScrolledWindow, TextMark};
use relm4::{send, ComponentUpdate, Components, Widgets};
use rustybrain_core::config::Config;
use rustybrain_core::md::TreeCursor;
use rustybrain_core::md::{Node, Tree};
use rustybrain_core::zettel::Zettel;

use self::block::Blocking;

pub enum Msg {
    Open(Zettel),
    Changed,
    Cursor,
    InsertText(TextMark, String),
}

pub struct Model {
    tree: Option<Tree>,
    zettel: Option<Zettel>,

    #[allow(dead_code)]
    config: Config,
    buffer: gtk::TextBuffer,
    title: gtk::EntryBuffer,
    view: gtk::TextView,

    #[allow(dead_code)]
    table: gtk::TextTagTable,
    blocks: Vec<block::Block>,
}

pub struct EditorComponents {}

impl Components<Model> for EditorComponents {
    fn init_components(
        _parent_model: &Model,
        _parent_sender: relm4::Sender<Msg>,
    ) -> Self {
        EditorComponents {}
    }

    fn connect_parent(&mut self, _parent_widgets: &Editor) {}
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Editor;

    type Components = EditorComponents;
}

pub struct Editor {
    window: gtk::ScrolledWindow,
    title: gtk::Entry,
}

impl Model {
    fn open_zettel(&mut self, z: Zettel) {
        self.buffer.set_text(z.content());
        self.title.set_text(z.title());
        self.zettel = Some(z);
    }

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

        if let Some(zettel) = self.zettel.as_mut() {
            zettel.set_content(&text);
        }

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
        let title = gtk::EntryBuffer::builder().build();

        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .vexpand(true)
            .hexpand(true)
            .pixels_inside_wrap(10)
            .wrap_mode(gtk::WrapMode::Char)
            .build();

        let mut model = Model {
            tree: None,
            zettel: None,
            config: parent_model.config.clone(),
            blocks: vec![],
            buffer,
            title,
            table,
            view,
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
            Msg::Open(z) => self.open_zettel(z),
            Msg::InsertText(m, s) => {
                if s == "@" {
                    let mut iter = self.buffer.iter_at_mark(&m);
                    let anchor = self.buffer.create_child_anchor(&mut iter);
                    let child =
                        gtk::Label::builder().label("You Complete Me").build();
                    self.view.add_child_at_anchor(&child, &anchor);
                }
            }
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
        _components: &EditorComponents,
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .margin_end(10)
            .build();

        let entry = gtk::Entry::builder()
            .hexpand(true)
            .vexpand(false)
            .placeholder_text("Title")
            .buffer(&model.title)
            .build();

        box_.append(&entry);
        box_.append(&model.view);

        let window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&box_)
            .build();

        let s = sender.clone();
        model
            .buffer
            .connect_changed(move |_| send!(s, Msg::Changed));

        let s = sender.clone();
        model.buffer.connect_insert_text(move |b, i, t| {
            send!(
                s,
                Msg::InsertText(b.create_mark(None, i, false), t.to_string())
            )
        });

        model.buffer.connect_cursor_position_notify(move |_| {
            send!(sender.clone(), Msg::Cursor)
        });

        Editor {
            window,
            title: entry,
        }
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        if model.title.text() == "" {
            self.title.set_placeholder_text(Some("Title"))
        } else {
            self.title.set_placeholder_text(None)
        }
    }
}
