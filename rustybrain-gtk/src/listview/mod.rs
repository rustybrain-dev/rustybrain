use gtk::prelude::*;
use gtk::Adjustment;
use gtk::ScrolledWindow;
use relm::Update;
use relm::Widget;
use relm_derive::Msg;

pub struct Model {}

#[derive(Msg)]
pub enum Msg {}

pub struct ListView {
    window: ScrolledWindow,
}

impl Update for ListView {
    type Model = Model;

    type ModelParam = ();

    type Msg = Msg;

    fn model(relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Self::Msg) {
        match event {}
    }
}

impl Widget for ListView {
    type Root = gtk::ScrolledWindow;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let window = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
        ListView { window }
    }
}
