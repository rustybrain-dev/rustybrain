mod style;

use gtk::prelude::*;
use relm::connect;
use relm::{Update, Widget};
use relm_derive::Msg;
use rustybrain_core::md::Tree;
use rustybrain_core::md::TreeCursor;

pub struct Model {
    tree: Option<Tree>,
}

#[derive(Msg)]
pub enum Msg {
    Changed,
}

pub struct Editor {
    model: Model,
    box_: gtk::Box,
    buffer: gtk::TextBuffer,
}

impl Editor {
    fn on_buffer_changed(&mut self) {
        println!("Changed!!");
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        let text = self.buffer.text(&start, &end, true);
        if text.is_none() {
            return;
        }

        if let Ok(tree) = rustybrain_core::md::parse(
            text.unwrap().as_str(),
            self.model.tree.as_ref(),
        ) {
            self.model.tree = tree;
        } else {
            self.model.tree = None;
        }
        if let Some(tree) = &self.model.tree {
            Self::walk(tree.walk());
        }
    }

    fn walk(cursor: TreeCursor) {}
}

impl Update for Editor {
    type Model = Model;

    type ModelParam = ();

    type Msg = Msg;

    fn model(relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model { tree: None }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Changed => self.on_buffer_changed(),
        }
    }
}

impl Widget for Editor {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.box_.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let tag_table = style::Style::new().table();
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let buffer = gtk::TextBuffer::new(Some(&tag_table));
        buffer.set_text("Nobody ever Start From Scratch");
        connect!(
            relm,
            buffer,
            connect_changed(_),
            return (Some(Msg::Changed), ())
        );

        let start = buffer.start_iter();
        let end = buffer.end_iter();
        buffer.apply_tag_by_name("h1", &start, &end);
        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .width_request(800)
            .height_request(600)
            .has_tooltip(true)
            .margin(10)
            .build();
        view.set_size_request(800, 600);
        box_.add(&view);
        Editor {
            model,
            box_,
            buffer,
        }
    }
}
