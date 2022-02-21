use gtk::{prelude::*, Label, ListBoxRow};
use relm4::{send, AppUpdate, ComponentUpdate};
use rustybrain_core::zettel::Zettel;

pub enum Msg {
    Activated,
}

pub struct Model {
    zettel: Zettel,
}

pub struct Item {
    row: ListBoxRow,

    #[allow(dead_code)]
    label: Label,
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

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Item;

    type Components = ();
}

impl ComponentUpdate<super::RowModel> for Model {
    fn init_model(parent_model: &super::RowModel) -> Self {
        Model {
            zettel: parent_model.zettel.clone(),
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Activated => send!(
                parent_sender,
                super::Msg::ZettelSelected(self.zettel.clone())
            ),
        }
    }
}

impl relm4::Widgets<Model, super::RowModel> for Item {
    type Root = ListBoxRow;

    fn init_view(
        model: &Model,
        _components: &(),
        _sender: relm4::Sender<Msg>,
    ) -> Self {
        let row = ListBoxRow::new();
        let label = Label::new(Some(model.zettel.title()));
        row.set_child(Some(&label));
        Item { row, label }
    }

    fn root_widget(&self) -> Self::Root {
        self.row.clone()
    }

    fn view(&mut self, _model: &Model, _sender: relm4::Sender<Msg>) {}
}
