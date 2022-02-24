use std::{cell::RefCell, rc::Rc};

use gdk::{Key, ModifierType};
use gtk::{
    prelude::*, ApplicationWindow, CallbackAction, Dialog, KeyvalTrigger,
    Shortcut, ShortcutController,
};
use relm4::{send, ComponentUpdate, Widgets};
use rustybrain_core::kasten::Kasten;

use crate::AppModel;

pub struct Model {
    dialog: Dialog,
    kasten: Option<Rc<RefCell<Kasten>>>,
}

pub enum Msg {
    Init(ApplicationWindow, Rc<RefCell<Kasten>>),
    Show,
    Changed(String),
}

pub struct Search {
    dialog: Dialog,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Search;

    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        Model {
            dialog: gtk::Dialog::builder()
                .destroy_with_parent(true)
                .decorated(true)
                .modal(true)
                .build(),
            kasten: None,
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
        model.dialog.set_child(Some(&entry));

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
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.dialog.clone()
    }

    fn view(&mut self, _model: &Model, _sender: relm4::Sender<Msg>) {}
}
