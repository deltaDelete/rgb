use crate::widgets::tray_item::TrayItem;
use gtk::prelude::{OrientableExt, WidgetExt};
use log::debug;
use relm4::component::AsyncComponentParts;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::SimpleAsyncComponent;
use relm4::AsyncComponentSender;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use system_tray::client::{Client, Event, UpdateEvent};
use system_tray::item::StatusNotifierItem;
use system_tray::menu::TrayMenu;

type State = HashMap<String, (StatusNotifierItem, Option<TrayMenu>)>;

pub struct SysTray {
    client: Rc<Client>,
    items: Arc<Mutex<State>>,
    tray_items: FactoryVecDeque<TrayItem>,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for SysTray {
    type Init = ();
    type Input = Event;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "module",
            add_css_class: "systray",
            set_orientation: gtk::Orientation::Horizontal,

            //TODO FACTORY [TrayItem]
            // gtk::Label {
            //     set_label: "Hello"
            // },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let client = Client::new("ru.deltadelete.regbar")
            .await
            .expect("Another program is serving system tray!");
        let client = Rc::new(client);
        let mut tray_rx = client.subscribe();

        let sender_clone = sender.clone();
        relm4::spawn(async move {
            while let Ok(ev) = tray_rx.recv().await {
                sender_clone.input(ev);
            }
        });

        let items = client.items();
        let sender_clone = sender.clone();
        if let Ok(guard) = items.lock() {
            let map: State = guard.clone();
            map.iter().for_each(|x| {
                let id = x.0.clone();
                let tuple = x.1;
                let item = Box::new(tuple.0.clone());
                sender_clone.input(Event::Add(id.clone(), item));
                if let Some(menu) = tuple.1.clone() {
                    sender_clone.input(Event::Update(id, UpdateEvent::Menu(menu)));
                }
            })
        }


        let tray_items = FactoryVecDeque::<TrayItem>::builder()
            .launch(root.clone())
            .detach();

        let model = Self {
            client,
            items,
            tray_items,
        };
        let root = model.tray_items.widget();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) -> () {
        match message {
            Event::Add(id, _item) => {
                debug!("Tray item added {}", id);
                let mut guard = self.tray_items.guard();
                let items = self.items.lock().unwrap();
                let id_clone = id.clone();
                let item = items.get(&id_clone).unwrap();
                let tray = item.clone();
                guard.push_back((id, tray, self.client.clone()));
            }
            Event::Update(id, update_event) => {
                debug!("Tray item updated {}", id);
                let guard = self.tray_items.guard();
                let item = guard.iter().position(|it| it.key == id);
                if let Some(option) = item {
                    guard.send(option, update_event);
                }
            }
            Event::Remove(id) => {
                debug!("Tray item removed {}", id);
                let mut guard = self.tray_items.guard();
                let some_index = guard.iter().position(|it| it.key == id);
                if let Some(index) = some_index {
                    _ = guard.remove(index);
                }
            }
        }
    }
}
