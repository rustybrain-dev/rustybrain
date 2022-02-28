use gtk::prelude::*;
use gtk::{MessageDialog, MessageType};
use relm4::{send, ComponentUpdate, Widgets};

use super::AppModel;

pub enum Msg {
    Show(gtk::MessageType, String),
    Hide,
}

pub struct Model {
    show: bool,
    title: String,
    type_: MessageType,
}

pub struct Message {
    dialog: MessageDialog,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Message;

    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        Model {
            show: false,
            title: "".to_string(),
            type_: MessageType::Error,
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
            Msg::Show(type_, title) => {
                self.type_ = type_;
                self.title = title;
                self.show = true;
            }
            Msg::Hide => self.show = false,
        }
    }
}

impl Widgets<Model, AppModel> for Message {
    type Root = MessageDialog;

    fn init_view(
        _model: &Model,
        _components: &(),
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let dialog = MessageDialog::builder().build();
        dialog.connect_response(move |_, _| send!(sender, Msg::Hide));
        Self { dialog }
    }

    fn root_widget(&self) -> Self::Root {
        self.dialog.clone()
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        self.dialog.set_message_type(model.type_);
        self.dialog.set_text(Some(&model.title));
        if model.show {
            self.dialog.show();
        } else {
            self.dialog.hide();
        }
    }
}
