use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::ListBox;
use gtk::ScrolledWindow;
use relm4::ComponentUpdate;
use relm4::Widgets;
use rustybrain_core::kasten::Kasten;
use rustybrain_core::zettel::Zettel;

use crate::AppModel;

pub struct Model {
    kasten: Rc<RefCell<Kasten>>,
    zettel: Option<Rc<RefCell<Zettel>>>,
}

pub enum Msg {
    ChangeZettel(Rc<RefCell<Zettel>>),
}

pub struct Backlinks {
    window: ScrolledWindow,
    layout: ListBox,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Backlinks;

    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(parent_model: &AppModel) -> Self {
        Model {
            kasten: parent_model.kasten.clone(),
            zettel: None,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        _parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::ChangeZettel(z) => self.zettel = Some(z),
        }
    }
}

impl Widgets<Model, super::AppModel> for Backlinks {
    type Root = gtk::ScrolledWindow;

    fn init_view(
        _model: &Model,
        _components: &(),
        _sender: relm4::Sender<Msg>,
    ) -> Self {
        let layout = ListBox::builder().build();
        let window = ScrolledWindow::builder()
            .width_request(200)
            .child(&layout)
            .build();
        Backlinks { window, layout }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        while let Some(c) = self.layout.last_child() {
            self.layout.remove(&c);
        }
        if let Some(z) = model.zettel.as_ref() {
            for item in model.kasten.borrow().iter_backlinks(&z.borrow()) {
                let z = &item.borrow();
                let label = gtk::Label::builder().label(z.title()).build();
                let row = gtk::ListBoxRow::builder().child(&label).build();
                self.layout.append(&row);
            }
        }
    }
}
