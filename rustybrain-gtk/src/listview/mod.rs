mod item;

use gtk::prelude::*;
use gtk::ListBox;
use gtk::ScrolledWindow;
use relm4::AppUpdate;
use relm4::ComponentUpdate;
use relm4::Components;
use relm4::RelmComponent;
use relm4::Widgets;
use rustybrain_core::config::Config;
use rustybrain_core::kasten::Kasten;
use rustybrain_core::zettel::Zettel;

pub struct Model {
    config: Config,
}

pub enum Msg {}

pub struct ListView {
    window: ScrolledWindow,

    #[allow(dead_code)]
    view: ListBox,
}

pub struct RowModel {
    zettel: Zettel,
}

impl relm4::Model for RowModel {
    type Msg = Msg;

    type Widgets = ListView;

    type Components = ();
}

pub struct ListViewComponents {
    rows: Vec<RelmComponent<item::Model, RowModel>>,
}

impl Components<Model> for ListViewComponents {
    fn init_components(
        parent_model: &Model,
        parent_sender: relm4::Sender<Msg>,
    ) -> Self {
        let mut items = vec![];

        let kasten = Kasten::new(parent_model.config.clone());
        for zettel in kasten {
            match zettel {
                Ok(z) => {
                    let model = RowModel { zettel: z };
                    let item =
                        RelmComponent::new(&model, parent_sender.clone());
                    items.push(item);
                }
                Err(e) => println!("error {:?}", e),
            }
        }
        ListViewComponents { rows: items }
    }

    fn connect_parent(&mut self, _parent_widgets: &ListView) {}
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = ListView;

    type Components = ListViewComponents;
}

impl AppUpdate for Model {
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: relm4::Sender<Self::Msg>,
    ) -> bool {
        true
    }
}

impl ComponentUpdate<super::AppModel> for Model {
    fn init_model(parent_model: &super::AppModel) -> Self {
        Model {
            config: parent_model.config.clone(),
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
    }
}

impl Widgets<Model, super::AppModel> for ListView {
    type Root = gtk::ScrolledWindow;

    fn init_view(
        model: &Model,
        components: &ListViewComponents,
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let view = ListBox::new();
        let window = ScrolledWindow::new();
        for item in &components.rows {
            view.append(item.root_widget());
        }
        window.set_child(Some(&view));
        window.set_width_request(200);
        ListView { window, view }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, model: &Model, sender: relm4::Sender<Msg>) {}
}
