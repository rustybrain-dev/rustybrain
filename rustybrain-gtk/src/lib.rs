mod backlinks;
mod editor;
mod listview;

use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::CssProvider;
use gtk::StyleContext;
use relm4::AppUpdate;
use relm4::Components;
use relm4::Model;
use relm4::RelmApp;
use relm4::RelmComponent;
use relm4::Widgets;
use rustybrain_core::config::Config;

pub enum Msg {
    Quit,
}

pub struct AppModel {
    config: Config,
}

pub struct AppComponents {
    editor: RelmComponent<editor::Model, AppModel>,
    listview: RelmComponent<listview::Model, AppModel>,
    backlinks: RelmComponent<backlinks::Model, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        parent_model: &AppModel,
        parent_sender: relm4::Sender<Msg>,
    ) -> Self {
        AppComponents {
            editor: RelmComponent::new(parent_model, parent_sender.clone()),
            listview: RelmComponent::new(parent_model, parent_sender.clone()),
            backlinks: RelmComponent::new(parent_model, parent_sender),
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
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        match msg {
            Msg::Quit => todo!(),
        }
    }
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = ApplicationWindow;

    fn init_view(
        _model: &AppModel,
        components: &AppComponents,
        _sender: relm4::Sender<Msg>,
    ) -> Self {
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .vexpand(true)
            .build();

        box_.append(components.listview.root_widget());
        box_.append(components.editor.root_widget());
        box_.append(components.backlinks.root_widget());

        let window = ApplicationWindow::builder().build();
        window.set_title(Some(
            "Rusty Brain -- To Help You Build Your Second Brain!",
        ));
        window.set_child(Some(&box_));
        window.set_default_size(1200, 800);

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
 * {
      background-color: red;
      border-color: shade (mix (rgb (34, 255, 120), #fff, 0.5), 0.9);
    }
"#;

pub fn run(config: &Config) {
    let model = AppModel {
        config: config.clone(),
    };
    let app = RelmApp::new(model);
    app.run();
}
