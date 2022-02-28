use std::{cell::RefCell, rc::Rc};

use gdk::{Key, ModifierType};
use gtk::{
    prelude::*, ApplicationWindow, CallbackAction, Dialog, KeyvalTrigger,
    Shortcut, ShortcutController,
};
use relm4::{send, ComponentUpdate, Widgets};
use rustybrain_core::{kasten::Kasten, zettel::Zettel};

use crate::AppModel;

pub struct Model {
    dialog: Dialog,
    zettels: Vec<Zettel>,
    kasten: Option<Rc<RefCell<Kasten>>>,
}

pub enum Msg {
    Init(ApplicationWindow, Rc<RefCell<Kasten>>),
    Show,
    Changed(String),
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
    fn init_model(parent_model: &AppModel) -> Self {
        let kasten = parent_model.kasten.borrow();
        let mut zettels = vec![];
        for z in kasten.iter() {
            let item = z.unwrap();
            zettels.push(item);
        }
        Model {
            dialog: gtk::Dialog::builder()
                .destroy_with_parent(true)
                .decorated(true)
                .modal(true)
                .build(),
            kasten: None,
            zettels,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &(),
        _sender: relm4::Sender<Self::Msg>,
        _parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Init(w, k) => {
                self.dialog.set_transient_for(Some(&w));
                self.kasten = Some(k);
            }
            Msg::Changed(s) => {
                if let Some(kasten) = &self.kasten {
                    let kasten = kasten.borrow();
                    kasten.search_title(&s).unwrap();
                }
            }
            Msg::Show => self.dialog.show(),
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
        let entry = gtk::SearchEntry::builder().build();
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let list_box = gtk::ListBox::builder().build();
        box_.append(&entry);
        box_.append(&list_box);
        model.dialog.set_child(Some(&box_));

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
        for item in model.zettels.iter() {
            let label = gtk::Label::builder().label(item.title()).build();
            let row = gtk::ListBoxRow::builder().child(&label).build();
            self.list_box.append(&row);
        }
    }
}
