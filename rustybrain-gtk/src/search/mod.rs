use std::{cell::RefCell, rc::Rc};

use gdk::{Key, ModifierType};
use gtk::{
    prelude::*, ApplicationWindow, Dialog, EventControllerKey, MessageType,
    ScrolledWindow,
};
use relm4::{send, ComponentUpdate, Widgets};
use rustybrain_core::{
    kasten::{Kasten, KastenError},
    zettel::Zettel,
};

use crate::AppModel;

pub struct Model {
    app_win: Option<ApplicationWindow>,
    zettels: Vec<Zettel>,
    searching: String,
    inserting: bool,
    show: bool,
    kasten: Option<Rc<RefCell<Kasten>>>,
}

pub enum Msg {
    Init(ApplicationWindow, Rc<RefCell<Kasten>>),
    Show,
    Hide,
    Changed(String),
    Search(Rc<RefCell<Kasten>>, String),
    Activate(Option<Zettel>),
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
            app_win: None,
            kasten: None,
            show: false,
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
            Msg::Show => self.show = true,
            Msg::Hide => self.show = false,
            Msg::Init(w, k) => {
                self.app_win = Some(w);
                self.handle_init(&k.borrow(), parent_sender);
                self.kasten = Some(k);
            }
            Msg::Changed(s) => {
                self.searching = s.clone();
                if let Some(kasten) = &self.kasten {
                    send!(sender, Msg::Search(kasten.clone(), s));
                }
            }
            Msg::Search(k, s) => {
                self.handle_search(&k.borrow(), parent_sender, &s)
            }
            Msg::Activate(item) => {
                send!(sender, Msg::Hide);
                if let Some(z) = item {
                    send!(parent_sender, super::Msg::ChangeZettel(z));
                } else {
                    send!(
                        parent_sender,
                        super::Msg::NewZettel(self.searching.to_string())
                    );
                }
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
                if set.is_empty() {
                    self.handle_init(kasten, parent_sender);
                    return;
                }
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
        _model: &Model,
        _components: &(),
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let dialog = gtk::Dialog::builder()
            .destroy_with_parent(true)
            .decorated(true)
            .modal(true)
            .build();
        let entry = gtk::SearchEntry::builder().hexpand(true).build();
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let list_box = gtk::ListBox::builder().build();
        let window = ScrolledWindow::builder()
            .hexpand(true)
            .height_request(200)
            .width_request(600)
            .child(&list_box)
            .build();
        box_.append(&entry);
        box_.append(&window);
        dialog.set_child(Some(&box_));

        let s = sender.clone();
        entry.connect_changed(move |e| {
            send!(s, Msg::Changed(e.text().as_str().to_string()))
        });
        let key_ctrl = EventControllerKey::new();
        let s = sender.clone();
        key_ctrl.connect_key_released(move |_, k, _, m| {
            if m == ModifierType::empty() && k == Key::Escape {
                send!(s, Msg::Hide);
            }
        });
        dialog.add_controller(&key_ctrl);

        Search { dialog, list_box }
    }

    fn root_widget(&self) -> Self::Root {
        self.dialog.clone()
    }

    fn view(&mut self, model: &Model, sender: relm4::Sender<Msg>) {
        self.dialog.set_transient_for(model.app_win.as_ref());
        if model.show {
            self.dialog.show();
        } else {
            self.dialog.hide();
        }
        loop {
            match self.list_box.last_child() {
                Some(c) => self.list_box.remove(&c),
                None => break,
            }
        }
        if model.searching != "" {
            self.list_box
                .append(&self.new_list_row(model, sender.clone()));
        }

        for item in model.zettels.iter() {
            if model.inserting {
                self.list_box.append(&self.row(
                    item.title(),
                    "Insert",
                    Some(item.clone()),
                    sender.clone(),
                ));
            } else {
                self.list_box.append(&self.row(
                    item.title(),
                    "Go",
                    Some(item.clone()),
                    sender.clone(),
                ));
            }
        }
    }
}

impl Search {
    fn row(
        &self,
        item: &str,
        btn_label: &str,
        zettel: Option<Zettel>,
        sender: relm4::Sender<Msg>,
    ) -> gtk::ListBoxRow {
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
        btn.connect_clicked(move |_| {
            send!(sender, Msg::Activate(zettel.clone()));
        });
        box_.append(&label);
        box_.append(&btn);
        gtk::ListBoxRow::builder().child(&box_).build()
    }

    fn new_list_row(
        &self,
        model: &Model,
        sender: relm4::Sender<Msg>,
    ) -> gtk::ListBoxRow {
        self.row(&model.searching, "New", None, sender)
    }
}
