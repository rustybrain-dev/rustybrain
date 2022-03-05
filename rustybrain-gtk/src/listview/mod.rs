mod item;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::ListBox;
use gtk::ListBoxRow;
use gtk::ScrolledWindow;
use relm4::send;
use relm4::ComponentUpdate;
use relm4::Components;
use relm4::RelmComponent;
use relm4::Widgets;
use rustybrain_core::kasten::Kasten;
use rustybrain_core::zettel::Zettel;

pub struct Model {
    kasten: Rc<RefCell<Kasten>>,
}

pub enum Msg {
    RowSelected(ListBoxRow),
    ZettelSelected(Rc<RefCell<Zettel>>),
}

pub struct ListView {
    window: ScrolledWindow,

    #[allow(dead_code)]
    view: ListBox,
}

pub struct RowModel {
    zettel: Rc<RefCell<Zettel>>,
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
        let kasten = parent_model.kasten.borrow();
        for zettel in kasten.iter() {
            let model = RowModel {
                zettel: zettel.clone(),
            };
            let item = RelmComponent::new(&model, parent_sender.clone());
            items.push(item);
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

impl ComponentUpdate<super::AppModel> for Model {
    fn init_model(parent_model: &super::AppModel) -> Self {
        Model {
            kasten: parent_model.kasten.clone(),
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::RowSelected(row) => {
                for item in &components.rows {
                    if item.root_widget() == &row {
                        send!(
                            item.sender(),
                            crate::listview::item::Msg::Activated
                        );
                    }
                }
            }
            Msg::ZettelSelected(zettel) => {
                send!(parent_sender, super::Msg::ChangeZettel(zettel))
            }
        }
    }
}

impl Widgets<Model, super::AppModel> for ListView {
    type Root = gtk::ScrolledWindow;

    fn init_view(
        _model: &Model,
        components: &ListViewComponents,
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let view = ListBox::new();
        for item in &components.rows {
            view.append(item.root_widget());
        }
        view.connect_row_selected(move |_, row| {
            if let Some(r) = row {
                send!(sender, Msg::RowSelected(r.clone()))
            }
        });

        let window = ScrolledWindow::new();
        window.set_child(Some(&view));
        window.set_width_request(200);
        ListView { window, view }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &Model, _sender: relm4::Sender<Msg>) {}
}
