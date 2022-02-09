use gtk::prelude::*;
use gtk::TextTagTable;
use relm::{Update, Widget};
use relm_derive::Msg;

pub struct Model {}

#[derive(Msg)]
pub enum Msg {
    Changed,
}

pub struct Editor {
    model: Model,
    box_: gtk::Box,
}

impl Update for Editor {
    type Model = Model;

    type ModelParam = ();

    type Msg = Msg;

    fn model(relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Changed => println!("changed"),
        }
    }
}

impl Widget for Editor {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.box_.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let buffer = gtk::TextBuffer::new::<TextTagTable>(None);
        buffer.set_text("Hello, RustyBrain!");
        let view = gtk::TextView::with_buffer(&buffer);
        view.set_size_request(800, 600);
        box_.add(&view);
        Editor { model, box_ }
    }
}
