use gtk::prelude::*;
use gtk::Adjustment;
use gtk::Label;
use gtk::ListBox;
use gtk::ListBoxRow;
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
    view: ListBox,
    rows: Vec<ListBoxRow>,
    labels: Vec<Label>,
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
        let view = ListBox::new();
        let mut rows = vec![];
        let mut labels = vec![];

        let kasten = Kasten::new(model.config.clone());
        for zettel in kasten {
            match zettel {
                Ok(z) => {
                    println!("{:?}", z);
                    let row = ListBoxRow::new();
                    let label = Label::new(Some(z.title()));
                    row.set_child(Some(&label));
                    view.add(&row);

                    rows.push(row);
                    labels.push(label);
                }
                Err(e) => println!("error {:?}", e),
            }
        }
        let window = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
        window.set_child(Some(&view));
        window.set_width_request(200);
        ListView {
            window,
            view,
            rows,
            labels,
        }
    }
}
