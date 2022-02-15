use gtk::{prelude::*, Label, ListBoxRow};
use relm::{Update, Widget};
use relm_derive::Msg;
use rustybrain_core::zettel::Zettel;

#[derive(Msg)]
pub enum Msg {}

pub struct Model {
    zettel: Zettel,
}

pub struct Item {
    row: ListBoxRow,

    #[allow(dead_code)]
    label: Label,
}

impl Update for Item {
    type Model = Model;

    type ModelParam = Zettel;

    type Msg = Msg;

    fn model(_relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model { zettel: param }
    }

    fn update(&mut self, _event: Self::Msg) {}
}

impl Widget for Item {
    type Root = ListBoxRow;

    fn root(&self) -> Self::Root {
        self.row.clone()
    }

    fn view(_relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let row = ListBoxRow::new();
        let label = Label::new(Some(model.zettel.title()));
        row.set_child(Some(&label));
        Item { row, label }
    }
}
