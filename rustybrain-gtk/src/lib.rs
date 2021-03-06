mod backlinks;
mod editor;
mod listview;
mod msg;
mod search;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::ApplicationWindow;
use gtk::CssProvider;
use gtk::MessageType;
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

#[derive(Clone, Debug)]
pub enum Msg {
    Quit,
    StartSearch,
    StartInsert,
    Init(ApplicationWindow),
    ChangeZettel(Rc<RefCell<Zettel>>),
    InsertZettel(Rc<RefCell<Zettel>>),
    /// Means insert current zettel to previous zettel after save.
    OpenZettelOnStack(Rc<RefCell<Zettel>>),
    NewZettel(String, bool),
    ShowMsg(MessageType, String),
}

pub struct AppModel {
    show_list: bool,
    show_back: bool,

    config: Rc<RefCell<Config>>,
    kasten: Rc<RefCell<Kasten>>,
}

pub struct AppComponents {
    editor: RelmComponent<editor::Model, AppModel>,
    listview: RelmComponent<listview::Model, AppModel>,
    backlinks: RelmComponent<backlinks::Model, AppModel>,
    search: RelmComponent<search::Model, AppModel>,
    msg: RelmComponent<msg::Model, AppModel>,
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
            msg: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &AppWidgets) {}
}

pub struct AppWidgets {
    window: ApplicationWindow,

    main_layout: gtk::Box,
    left: gtk::ScrolledWindow,
    center: gtk::Box,
    right: gtk::ScrolledWindow,
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
        sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        match msg {
            Msg::Quit => relm4::gtk_application().quit(),
            Msg::ChangeZettel(z) => {
                send!(components.editor.sender(), editor::Msg::Open(z.clone()));
                send!(
                    components.backlinks.sender(),
                    backlinks::Msg::ChangeZettel(z)
                );
            }
            Msg::InsertZettel(z) => {
                send!(components.editor.sender(), editor::Msg::Insert(z))
            }
            Msg::OpenZettelOnStack(z) => {
                send!(components.editor.sender(), editor::Msg::OpenOnStack(z))
            }
            Msg::Init(w) => {
                send!(
                    components.search.sender(),
                    search::Msg::Init(w, self.kasten.clone())
                )
            }
            Msg::StartSearch => {
                send!(components.search.sender(), search::Msg::Show(false))
            }
            Msg::StartInsert => {
                send!(components.search.sender(), search::Msg::Show(true))
            }
            Msg::ShowMsg(t, s) => {
                send!(components.msg.sender(), msg::Msg::Show(t, s))
            }
            Msg::NewZettel(title, inserting) => {
                match self.kasten.borrow_mut().create(&title) {
                    Ok(z) => {
                        if inserting {
                            send!(sender, Msg::OpenZettelOnStack(z));
                        } else {
                            send!(sender, Msg::ChangeZettel(z));
                        }
                    }
                    Err(e) => send!(
                        sender,
                        Msg::ShowMsg(
                            MessageType::Error,
                            format!("Create note failed: {:?}!", e)
                        )
                    ),
                }
            }
        }
        true
    }
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = ApplicationWindow;

    fn init_view(
        model: &AppModel,
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

        let left = components.listview.root_widget().clone();
        let center = components.editor.root_widget().clone();
        let right = components.backlinks.root_widget().clone();

        window.set_child(Some(&box_));

        let shortcut_ctrl = ShortcutController::builder()
            .scope(gtk::ShortcutScope::Global)
            .build();

        let c = (*model.config).borrow();
        shortcut_ctrl.add_shortcut(&Self::bind_key(
            sender.clone(),
            c.shortcut().find(),
            Msg::StartSearch,
        ));
        shortcut_ctrl.add_shortcut(&Self::bind_key(
            sender.clone(),
            c.shortcut().insert(),
            Msg::StartInsert,
        ));
        shortcut_ctrl.add_shortcut(&Self::bind_key(
            sender.clone(),
            c.shortcut().quit(),
            Msg::Quit,
        ));
        window.add_controller(&shortcut_ctrl);
        window.connect_show(move |_| send!(sender, Msg::StartSearch));

        AppWidgets {
            window,
            main_layout: box_,
            left,
            right,
            center,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, model: &AppModel, _sender: relm4::Sender<Msg>) {
        while let Some(c) = self.main_layout.last_child() {
            self.main_layout.remove(&c);
        }
        if model.show_list {
            self.main_layout.append(&self.left);
        }
        self.main_layout.append(&self.center);
        if model.show_back {
            self.main_layout.append(&self.right);
        }

        let provider = CssProvider::new();
        provider.load_from_resource("/dev/rustybrain/app/assets/css/main.css");
        StyleContext::add_provider_for_display(
            &self.root_widget().display(),
            &provider,
            100,
        );
    }
}

impl AppWidgets {
    fn bind_key(sender: relm4::Sender<Msg>, key: &str, msg: Msg) -> Shortcut {
        let action = CallbackAction::new(move |_, _| {
            send!(sender, msg.clone());
            true
        });
        let trigger = ShortcutTrigger::parse_string(key).unwrap();
        Shortcut::builder()
            .trigger(&trigger)
            .action(&action)
            .build()
    }
}

pub fn run(config: Rc<RefCell<Config>>) {
    gio::resources_register_include!("app.gresource").unwrap();

    let model = AppModel {
        show_list: false,
        show_back: true,
        config: config.clone(),
        kasten: Rc::new(RefCell::new(Kasten::new(config).unwrap())),
    };
    let app = RelmApp::new(model);
    app.run();
}
