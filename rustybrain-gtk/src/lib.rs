mod editor;

use gtk::prelude::*;
use gtk::{Inhibit, Window, WindowType};
use relm::{Component, Relm, Update, Widget, connect};
use relm_derive::Msg;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

pub struct Model {}

pub struct Win {
    model: Model,
    window: Window,
	// Hold editor to avoid it been dropped, otherwise that will cause panic.
	// See also: https://github.com/antoyo/relm/issues/278
	editor: Component<editor::Editor>,
}

impl Update for Win {
    type Model = Model;

    type ModelParam = ();

    type Msg = Msg;

    fn model(relm: &Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let editor = relm::init::<editor::Editor>(()).unwrap();

        let window = Window::new(WindowType::Toplevel);
        window.set_title("Rusty Brain -- To Help You Build Your Second Brain!");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );
        window.set_position(gtk::WindowPosition::Mouse);
        window.set_child(Some(editor.widget()));
        window.resize(800, 600);
        window.show_all();
        Win { model, window, editor }
    }
}

pub fn run() {
    Win::run(()).unwrap()
}
