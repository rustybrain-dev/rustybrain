use gtk::prelude::*;
use gtk::Adjustment;
use gtk::ScrolledWindow;
use relm::Update;
use relm::Widget;
use relm_derive::Msg;
use rustybrain_core::config::Config;
use rustybrain_core::kasten::Kasten;

pub struct Model {
    config: Config,
}

#[derive(Msg)]
pub enum Msg {}

pub struct ListView {
    window: ScrolledWindow,
}

impl Update for ListView {
    type Model = Model;

    type ModelParam = Config;

    type Msg = Msg;

    fn model(_relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model { config: param }
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

    fn view(_relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let kasten = Kasten::new(model.config.clone());
        for zettel in kasten {
            match zettel {
                Ok(z) => println!("{:?}", z),
                Err(e) => println!("error {:?}", e),
            }
        }
        let window = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
        window.set_width_request(200);
        ListView { window }
    }
}
