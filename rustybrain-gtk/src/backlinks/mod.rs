use gtk::prelude::*;
use gtk::ScrolledWindow;
use relm4::AppUpdate;
use relm4::ComponentUpdate;
use relm4::Widgets;
use rustybrain_core::config::Config;

use crate::AppModel;

pub struct Model {
    #[allow(dead_code)]
    config: Config,
}

pub enum Msg {}

pub struct Backlinks {
    window: ScrolledWindow,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Backlinks;

    type Components = ();
}

impl AppUpdate for Model {
    fn update(
        &mut self,
        _msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        true
    }
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(parent_model: &AppModel) -> Self {
        Model {
            config: parent_model.config.clone(),
        }
    }

    fn update(
        &mut self,
        _msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        _parent_sender: relm4::Sender<super::Msg>,
    ) {
    }
}

impl Widgets<Model, super::AppModel> for Backlinks {
    type Root = gtk::ScrolledWindow;

    fn init_view(
        _model: &Model,
        _components: &(),
        _sender: relm4::Sender<Msg>,
    ) -> Self {
        let window = ScrolledWindow::new();
        window.set_width_request(200);
        Backlinks { window }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &Model, _sender: relm4::Sender<Msg>) {}
}
