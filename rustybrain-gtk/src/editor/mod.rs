mod block;
mod style;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    prelude::*, ActionBar, EventControllerFocus, MessageType, ScrolledWindow,
    TextTagTable, TextView,
};
use relm4::{send, ComponentUpdate, Components, Widgets};
use rustybrain_core::kasten::Kasten;
use rustybrain_core::md::{Node, Tree};
use rustybrain_core::zettel::Zettel;

use self::block::Blocking;

pub enum Msg {
    Open(Zettel),
    Insert(Zettel),
    OpenOnStack(Zettel),
    Changed,
    Save,
    Cursor,
    EditTitle,
    DoneEditTitle,
}

/// Zettel that be editing.
pub struct EditingZettel {
    title: gtk::EntryBuffer,
    buffer: gtk::TextBuffer,
    zettel: Zettel,

    view: TextView,
    blocks: Vec<block::Block>,

    #[allow(dead_code)]
    table: TextTagTable,
}

impl EditingZettel {
    fn new(zettel: Zettel, view: TextView) -> Self {
        let table = style::Style::new().table();
        let buffer = gtk::TextBuffer::builder()
            .enable_undo(true)
            .tag_table(&table)
            .build();
        let title = gtk::EntryBuffer::builder().build();

        buffer.set_text(zettel.content());
        title.set_text(zettel.title());

        let mut r = Self {
            buffer,
            title,
            zettel,
            table,
            view,

            blocks: vec![],
        };
        r.on_buffer_changed();
        r
    }

    fn listen_buffer_event(&self, sender: relm4::Sender<Msg>) {
        let s = sender.clone();

        self.buffer.connect_changed(move |_| send!(s, Msg::Changed));

        let s = sender.clone();
        self.buffer
            .connect_cursor_position_notify(move |_| send!(s, Msg::Cursor));
    }

    fn on_buffer_changed(&mut self) {
        while let Some(blk) = self.blocks.pop() {
            blk.umount(&self.view, &self.buffer);
        }

        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();

        self.buffer.remove_all_tags(&start, &end);
        self.buffer.apply_tag_by_name("p", &start, &end);

        let text = self.buffer.text(&start, &end, true);
        if let Err(_) = self.zettel.set_content(text.as_str()) {
            return;
        };

        let tree = self.zettel.tree();
        if self.zettel.tree().is_none() {
            return;
        }
        let t = tree.unwrap().clone();
        self.walk(&t);
    }

    fn walk(&mut self, tree: &Tree) {
        let mut cursor = tree.walk();
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
        let blk = block::Block::from_node(node, &self.buffer);
        blk.mount(&self.view, &self.buffer);
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
                blk.cursor_in(&self.view, &self.buffer)
            } else {
                blk.cursor_out(&self.view, &self.buffer)
            }
        }
    }

    fn insert_zettel_at_cursor(&self, z: &Zettel) {
        self.buffer
            .insert_at_cursor(&format!("[{}]({})", z.title(), z.zid(),));
    }

    fn save(
        &mut self,
        kasten: &mut Kasten,
        parent_sender: relm4::Sender<super::Msg>,
    ) -> bool {
        if !self.buffer.is_modified() {
            return false;
        }

        // TODO set when title is changed
        let title = self.title.text();
        self.zettel.set_title(&title);

        if let Err(err) = kasten.save(&self.zettel) {
            send!(
                parent_sender,
                super::Msg::ShowMsg(
                    MessageType::Error,
                    format!("Save note failed: {:?}", err)
                )
            );
        }
        self.buffer.set_modified(false);
        return true;
    }
}

pub struct Model {
    kasten: Rc<RefCell<Kasten>>,
    stack: Vec<EditingZettel>,

    view: gtk::TextView,
    editing_title: bool,
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
    layout: gtk::Box,
    main_win: gtk::ScrolledWindow,
    title_in: gtk::Entry,
    title_label: gtk::Label,
    title_show: gtk::Box,
    action_bar: gtk::ActionBar,
    save_btn: gtk::Button,
}

impl Model {
    fn open_zettel(&mut self, zettel: Zettel, sender: relm4::Sender<Msg>) {
        loop {
            if self.stack.pop().is_none() {
                break;
            }
        }

        self.open_zettel_on_stack(zettel, sender);
    }

    fn open_zettel_on_stack(
        &mut self,
        zettel: Zettel,
        sender: relm4::Sender<Msg>,
    ) {
        let ez = EditingZettel::new(zettel, self.view.clone());
        ez.listen_buffer_event(sender);
        self.stack.push(ez);
    }

    fn pop_stack_and_insert(&mut self, sender: relm4::Sender<Msg>) {
        if let Some(z) = self.stack.pop() {
            send!(sender, Msg::Insert(z.zettel));
        }
    }

    fn on_buffer_changed(&mut self) {
        if let Some(z) = self.stack.last_mut() {
            z.buffer.set_modified(true);
            z.on_buffer_changed();
        }
    }

    fn on_cursor_notify(&mut self) {
        if let Some(z) = self.stack.last_mut() {
            z.on_cursor_notify();
        }
    }

    fn insert_zettel_at_cursor(&self, zettel: &Zettel) {
        if let Some(z) = self.stack.last() {
            z.insert_zettel_at_cursor(zettel);
        }
    }

    fn save(&mut self, parent_sender: relm4::Sender<super::Msg>) -> bool {
        match self.stack.last_mut() {
            Some(z) => z.save(&mut self.kasten.borrow_mut(), parent_sender),
            None => false,
        }
    }
}

impl ComponentUpdate<super::AppModel> for Model {
    fn init_model(parent_model: &super::AppModel) -> Self {
        let view = gtk::TextView::builder()
            .vexpand(true)
            .hexpand(true)
            .pixels_inside_wrap(10)
            .wrap_mode(gtk::WrapMode::Char)
            .build();

        Model {
            kasten: parent_model.kasten.clone(),
            stack: vec![],
            editing_title: false,
            view,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Changed => self.on_buffer_changed(),
            Msg::Cursor => self.on_cursor_notify(),
            Msg::Open(z) => {
                self.editing_title = false;
                self.open_zettel(z, sender)
            }

            Msg::OpenOnStack(z) => {
                self.editing_title = false;
                self.open_zettel_on_stack(z, sender);
            }
            Msg::Insert(z) => {
                self.insert_zettel_at_cursor(&z);
            }
            Msg::Save => {
                if self.save(parent_sender) || self.stack.len() > 1 {
                    self.pop_stack_and_insert(sender);
                }
            }
            Msg::EditTitle => self.editing_title = true,
            Msg::DoneEditTitle => {
                self.editing_title = false;
            }
        }
    }
}

impl Widgets<Model, super::AppModel> for Editor {
    type Root = gtk::Box;

    fn root_widget(&self) -> Self::Root {
        self.layout.clone()
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

        let label = gtk::Label::builder().hexpand(true).vexpand(false).build();

        let entry = gtk::Entry::builder()
            .hexpand(true)
            .vexpand(false)
            .placeholder_text("Title")
            .build();

        let focus_ctrl = EventControllerFocus::builder().build();
        let s = sender.clone();
        focus_ctrl.connect_leave(move |_| send!(s, Msg::DoneEditTitle));
        entry.add_controller(&focus_ctrl);

        let window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .child(&model.view)
            .build();

        let title_show = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .vexpand(false)
            .build();
        let edit_btn = gtk::Button::builder().label("Edit").build();
        let s = sender.clone();
        edit_btn.connect_clicked(move |_| send!(s, Msg::EditTitle));
        title_show.append(&label);
        title_show.append(&edit_btn);

        let action_bar = ActionBar::builder().build();
        let save_btn = gtk::Button::builder().label("Save").build();
        let s = sender.clone();
        save_btn.connect_clicked(move |_| send!(s, Msg::Save));
        action_bar.pack_end(&save_btn);

        Editor {
            layout: box_,
            title_in: entry,
            title_label: label,
            title_show,
            main_win: window,
            action_bar,
            save_btn,
        }
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        loop {
            match self.layout.last_child() {
                Some(c) => self.layout.remove(&c),
                None => break,
            }
        }
        if model.editing_title {
            self.layout.append(&self.title_in);
        } else {
            self.layout.append(&self.title_show);
        }
        self.layout.append(&self.action_bar);
        self.layout.append(&self.main_win);

        if let Some(ez) = model.stack.last() {
            model.view.set_buffer(Some(&ez.buffer));
            self.title_in.set_buffer(&ez.title);

            if ez.buffer.is_modified() || model.stack.len() > 1 {
                self.save_btn.set_sensitive(true);
            } else {
                self.save_btn.set_sensitive(false);
            }
            self.title_label.set_text(&ez.title.text());

            if ez.title.text() == "" {
                self.title_in.set_placeholder_text(Some("Title"))
            } else {
                self.title_in.set_placeholder_text(None)
            }
        }
        model.view.grab_focus();
    }
}
