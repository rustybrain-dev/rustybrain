use std::{cell::RefCell, rc::Rc};

use gdk::{Key, ModifierType};
use gtk::{
    prelude::*, ApplicationWindow, CallbackAction, Dialog, KeyvalTrigger,
    MessageType, ScrolledWindow, Shortcut, ShortcutController,
};
use relm4::{send, ComponentUpdate, Widgets};
use rustybrain_core::{
    kasten::{Kasten, KastenError},
    zettel::Zettel,
};

use crate::AppModel;

pub struct Model {
    dialog: Dialog,
    zettels: Vec<Zettel>,
    searching: String,
    inserting: bool,
    kasten: Option<Rc<RefCell<Kasten>>>,
}

pub enum Msg {
    Init(ApplicationWindow, Rc<RefCell<Kasten>>),
    Show,
    Changed(String),
    Search(Rc<RefCell<Kasten>>, String),
}

pub struct Search {
    dialog: Dialog,
    list_box: gtk::ListBox,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Search;

    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        let zettels = vec![];
        Model {
            dialog: gtk::Dialog::builder()
                .destroy_with_parent(true)
                .decorated(true)
                .modal(true)
                .build(),
            kasten: None,
            searching: "".to_string(),
            inserting: false,
            zettels,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &(),
        sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Init(w, k) => {
                self.dialog.set_transient_for(Some(&w));
                self.handle_init(&k.borrow(), parent_sender);
                self.kasten = Some(k);
            }
            Msg::Changed(s) => {
                self.searching = s.clone();
                if let Some(kasten) = &self.kasten {
                    send!(sender, Msg::Search(kasten.clone(), s));
                }
            }
            Msg::Show => self.dialog.show(),
            Msg::Search(k, s) => {
                self.handle_search(&k.borrow(), parent_sender, &s)
            }
        }
    }
}

impl Model {
    fn handle_init(
        &mut self,
        kasten: &Kasten,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        for item in kasten.iter() {
            self.push_zettel(item, parent_sender.clone());
        }
    }

    fn handle_search(
        &mut self,
        kasten: &Kasten,
        parent_sender: relm4::Sender<super::Msg>,
        s: &str,
    ) {
        self.zettels.clear();
        match kasten.search_title(&s) {
            Ok(set) => {
                for item in kasten.iter() {
                    match item {
                        Ok(z) => {
                            let p = z.path().to_str().unwrap();
                            if set.contains::<String>(&p.to_string()) {
                                self.zettels.push(z);
                            }
                        }
                        Err(_) => send!(
                            parent_sender,
                            super::Msg::ShowMsg(
                                MessageType::Error,
                                "Load notes from slip-box failed!".to_string()
                            )
                        ),
                    }
                }
            }
            Err(_) => send!(
                parent_sender,
                super::Msg::ShowMsg(
                    MessageType::Error,
                    "Search notes from slip-box failed!".to_string()
                )
            ),
        };
    }

    fn push_zettel(
        &mut self,
        item: Result<Zettel, KastenError>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match item {
            Ok(z) => self.zettels.push(z),
            Err(_) => send!(
                parent_sender,
                super::Msg::ShowMsg(
                    MessageType::Error,
                    "Load notes from slip-box failed!".to_string()
                )
            ),
        }
    }
}

impl Widgets<Model, AppModel> for Search {
    type Root = Dialog;

    fn init_view(
        model: &Model,
        _components: &(),
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let entry = gtk::SearchEntry::builder().hexpand(true).build();
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let window = ScrolledWindow::builder()
            .hexpand(true)
            .height_request(200)
            .width_request(600)
            .child(&box_)
            .build();
        let list_box = gtk::ListBox::builder().build();
        box_.append(&entry);
        box_.append(&list_box);
        model.dialog.set_child(Some(&window));

        let trigger = KeyvalTrigger::new(Key::Escape, ModifierType::empty());
        let d = model.dialog.clone();
        let action = CallbackAction::new(move |_, _| {
            d.close();
            true
        });
        let shortcut = Shortcut::builder()
            .trigger(&trigger)
            .action(&action)
            .build();
        let ctrl = ShortcutController::builder()
            .scope(gtk::ShortcutScope::Managed)
            .build();
        ctrl.add_shortcut(&shortcut);
        model.dialog.add_controller(&ctrl);
        entry.connect_changed(move |e| {
            send!(sender, Msg::Changed(e.text().as_str().to_string()))
        });

        Search {
            dialog: model.dialog.clone(),
            list_box,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.dialog.clone()
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        loop {
            match self.list_box.last_child() {
                Some(c) => self.list_box.remove(&c),
                None => break,
            }
        }
        if model.searching != "" {
            self.list_box.append(&self.new_list_row(model));
        }

        for item in model.zettels.iter() {
            if model.inserting {
                self.list_box.append(&self.row(item.title(), "Insert"));
            } else {
                self.list_box.append(&self.row(item.title(), "Go"));
            }
        }
    }
}

impl Search {
    fn row(&self, item: &str, btn_label: &str) -> gtk::ListBoxRow {
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .build();
        let label = gtk::Label::builder()
            .label(item)
            .justify(gtk::Justification::Left)
            .hexpand(true)
            .build();

        let btn = gtk::Button::builder().label(btn_label).build();
        box_.append(&label);
        box_.append(&btn);
        gtk::ListBoxRow::builder().child(&box_).build()
    }

    fn new_list_row(&self, model: &Model) -> gtk::ListBoxRow {
        self.row(&model.searching, "New")
    }
}
