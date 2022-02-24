mod backlinks;
mod editor;
mod listview;
mod search;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::ApplicationWindow;
use gtk::CssProvider;
use gtk::StyleContext;
use gtk::{
    prelude::*, CallbackAction, Shortcut, ShortcutController, ShortcutTrigger,
};
use relm4::send;
use relm4::AppUpdate;
use relm4::Components;
use relm4::Model;
use relm4::RelmApp;
use relm4::RelmComponent;
use relm4::Widgets;
use rustybrain_core::config::Config;
use rustybrain_core::kasten::Kasten;
use rustybrain_core::zettel::Zettel;

pub enum Msg {
    Quit,
    StartSearch,
    Init(ApplicationWindow),
    ChangeZettel(Zettel),
}

pub struct AppModel {
    config: Config,
    kasten: Rc<RefCell<Kasten>>,
}

pub struct AppComponents {
    editor: RelmComponent<editor::Model, AppModel>,
    listview: RelmComponent<listview::Model, AppModel>,
    backlinks: RelmComponent<backlinks::Model, AppModel>,
    search: RelmComponent<search::Model, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        parent_model: &AppModel,
        parent_sender: relm4::Sender<Msg>,
    ) -> Self {
        AppComponents {
            editor: RelmComponent::new(parent_model, parent_sender.clone()),
            listview: RelmComponent::new(parent_model, parent_sender.clone()),
            backlinks: RelmComponent::new(parent_model, parent_sender.clone()),
            search: RelmComponent::new(parent_model, parent_sender.clone()),
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &AppWidgets) {}
}

pub struct AppWidgets {
    window: ApplicationWindow,

    #[allow(dead_code)]
    box_: gtk::Box,
}

impl Model for AppModel {
    type Msg = Msg;

    type Widgets = AppWidgets;

    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        match msg {
            Msg::Quit => todo!(),
            Msg::ChangeZettel(z) => {
                send!(components.editor.sender(), editor::Msg::Open(z))
            }
            Msg::Init(w) => {
                send!(
                    components.search.sender(),
                    search::Msg::Init(w, self.kasten.clone())
                )
            }
            Msg::StartSearch => {
                send!(components.search.sender(), search::Msg::Show)
            }
        }
        true
    }
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = ApplicationWindow;

    fn init_view(
        _model: &AppModel,
        components: &AppComponents,
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let window = ApplicationWindow::builder().build();
        window.set_title(Some(
            "Rusty Brain -- To Help You Build Your Second Brain!",
        ));
        window.set_default_size(1200, 800);
        send!(sender, Msg::Init(window.clone()));

        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .vexpand(true)
            .build();

        box_.append(components.listview.root_widget());
        box_.append(components.editor.root_widget());
        box_.append(components.backlinks.root_widget());

        window.set_child(Some(&box_));

        let sender = components.search.sender();
        let action = CallbackAction::new(move |_, _| {
            send!(sender, search::Msg::Show);
            true
        });
        let trigger = ShortcutTrigger::parse_string("<Control>i").unwrap();
        let shortcut = Shortcut::builder()
            .trigger(&trigger)
            .action(&action)
            .build();

        let shortcut_ctrl = ShortcutController::builder()
            .scope(gtk::ShortcutScope::Global)
            .build();

        shortcut_ctrl.add_shortcut(&shortcut);
        window.add_controller(&shortcut_ctrl);

        AppWidgets { window, box_ }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &AppModel, _sender: relm4::Sender<Msg>) {
        let provider = CssProvider::new();
        provider.load_from_data(CSS.as_bytes());
        StyleContext::add_provider_for_display(
            &self.root_widget().display(),
            &provider,
            100,
        );
    }
}

const CSS: &'static str = r#"
body {
      background-color: #fefefe;
    }
"#;

pub fn run(config: &Config) {
    let model = AppModel {
        config: config.clone(),
        kasten: Rc::new(RefCell::new(Kasten::new(config.clone()).unwrap())),
    };
    let app = RelmApp::new(model);
    app.run();
}
