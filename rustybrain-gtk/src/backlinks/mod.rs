use gtk::prelude::*;
use gtk::Adjustment;
use gtk::ScrolledWindow;
use relm::Update;
use relm::Widget;
use relm_derive::Msg;
use rustybrain_core::config::Config;

pub struct Model {}

#[derive(Msg)]
pub enum Msg {}

pub struct Backlinks {
    window: ScrolledWindow,
}

impl Update for Backlinks {
    type Model = Model;

    type ModelParam = Config;

    type Msg = Msg;

    fn model(
        _relm: &relm::Relm<Self>,
        _param: Self::ModelParam,
    ) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Self::Msg) {
        match event {}
    }
}

impl Widget for Backlinks {
    type Root = gtk::ScrolledWindow;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(_relm: &relm::Relm<Self>, _model: Self::Model) -> Self {
        let window = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
        window.set_width_request(200);
        Backlinks { window }
    }
}
