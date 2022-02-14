mod backlinks;
mod editor;
mod listview;

use gtk::prelude::*;
use gtk::CssProvider;
use gtk::StyleContext;
use gtk::{Inhibit, Window, WindowType};
use relm::{connect, Component, Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

pub struct Model {}

pub struct Win {
    #[allow(dead_code)]
    model: Model,
    window: Window,

    #[allow(dead_code)]
    box_: gtk::Box,

    // Hold editor to avoid it been dropped, otherwise that will cause panic.
    // See also: https://github.com/antoyo/relm/issues/278
    #[allow(dead_code)]
    editor: Component<editor::Editor>,

    #[allow(dead_code)]
    listview: Component<listview::ListView>,

    #[allow(dead_code)]
    backlinks: Component<backlinks::Backlinks>,

    #[allow(dead_code)]
    css_provider: CssProvider,

    #[allow(dead_code)]
    css_context: StyleContext,
}

impl Update for Win {
    type Model = Model;

    type ModelParam = ();

    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _paramm: Self::ModelParam) -> Self::Model {
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
        let css_provider = CssProvider::new();
        css_provider.load_from_data(CSS.as_bytes()).unwrap();
        let context = gtk::StyleContext::new();
        context.add_provider(
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        let listview = relm::init::<listview::ListView>(()).unwrap();
        let editor = relm::init::<editor::Editor>(()).unwrap();
        let backlinks = relm::init::<backlinks::Backlinks>(()).unwrap();
        box_.pack_start(listview.widget(), false, true, 2);
        box_.pack_start(editor.widget(), true, true, 2);
        box_.pack_end(backlinks.widget(), false, true, 2);

        let window = Window::new(WindowType::Toplevel);
        window.set_title("Rusty Brain -- To Help You Build Your Second Brain!");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );
        window.set_position(gtk::WindowPosition::Mouse);
        window.set_child(Some(&box_));
        window.resize(1200, 600);
        window.show_all();

        Win {
            model,
            window,
            box_,
            editor,
            listview,
            backlinks,
            css_provider,
            css_context: context,
        }
    }

    fn init_view(&mut self) {}
}

const CSS: &'static str = r#"
 * {
      background-color: red;
      border-color: shade (mix (rgb (34, 255, 120), #fff, 0.5), 0.9);
    }
"#;

pub fn run() {
    Win::run(()).unwrap()
}
