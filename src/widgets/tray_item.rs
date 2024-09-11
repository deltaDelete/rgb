use crate::ext::ParentFor;
use gtk::gdk_pixbuf::Colorspace;
use gtk::prelude::ButtonExt;
use gtk::prelude::PopoverExt;
use gtk::prelude::WidgetExt;
use gtk::prelude::{ActionMapExt, GestureSingleExt};
use gtk::{gdk, gio, glib};
use log::{debug, error};
use relm4::factory::FactoryComponent;
use relm4::FactorySender;
use std::rc::Rc;
use system_tray::client::{ActivateRequest, Client, UpdateEvent};
use system_tray::item::StatusNotifierItem;
use system_tray::menu::{MenuItem, MenuType, ToggleState, ToggleType, TrayMenu};

#[derive(Debug, Clone)]
pub struct TrayItem {
    pub key: String,
    item: StatusNotifierItem,
    menu: Option<TrayMenu>,
    client: Rc<Client>,
}

impl From<(String, (StatusNotifierItem, Option<TrayMenu>), Rc<Client>)> for TrayItem {
    fn from(value: (String, (StatusNotifierItem, Option<TrayMenu>), Rc<Client>)) -> Self {
        Self {
            key: value.0,
            item: value.1 .0,
            menu: value.1 .1,
            client: value.2,
        }
    }
}

impl
    From<(
        &String,
        &(StatusNotifierItem, Option<TrayMenu>),
        &Rc<Client>,
    )> for TrayItem
{
    fn from(
        value: (
            &String,
            &(StatusNotifierItem, Option<TrayMenu>),
            &Rc<Client>,
        ),
    ) -> Self {
        Self {
            key: value.0.clone(),
            item: value.1 .0.clone(),
            menu: value.1 .1.clone(),
            client: value.2.clone(),
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for TrayItem {
    type Input = system_tray::client::UpdateEvent;
    type Output = ();
    type Init = (String, (StatusNotifierItem, Option<TrayMenu>), Rc<Client>);
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "root"]
        gtk::Button {
            add_css_class: "systray__icon",
            #[watch]
            set_tooltip_text: self.item.title.as_deref(),
            #[watch]
            set_icon_name: self.item.icon_name.as_deref().unwrap_or("icon"),
            set_parent_for: popover = &gtk::PopoverMenu::from_model(self.build_model().as_ref()),
            connect_clicked: glib::clone!(
                #[strong(rename_to = key)]
                self.key,
                #[strong(rename_to = menu)]
                self.item.menu,
                #[strong(rename_to = client)]
                self.client,
                move |_btn| {
                    let key = key.clone();
                    let menu = menu.clone();
                    let client = client.clone();
                    relm4::spawn_local(async move {
                        let key_dbg = key.clone();
                        let rq = ActivateRequest {
                            address: key,
                            menu_path: menu.unwrap(),
                            submenu_id: 0,
                        };
                        match client.activate(rq).await {
                            Ok(()) => {
                                debug!("Sent activate request to {key_dbg}!")
                            }
                            Err(err) => {
                                error!("{err}")
                            }
                        };
                    });
                }),

            add_controller = gtk::GestureClick::new() {
                set_button: gdk::BUTTON_SECONDARY,
                connect_pressed[popover] => move |_, _, _, _| {
                    popover.popup();
                }
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, sender: FactorySender<Self>) -> Self {
        let model = Self::from(init);
        {
            let icon = model.item.icon_name.clone();
            let icon_theme_path = model.item.icon_theme_path.clone();
            if let Some(icon_theme_path) = icon_theme_path {
                gtk::IconTheme::for_display(&gdk::Display::default().unwrap())
                    .add_search_path(icon_theme_path);
            }
            sender.input(UpdateEvent::Icon(icon));
        }
        model
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: FactorySender<Self>,
    ) {
        match message {
            UpdateEvent::Icon(icon) => {
                self.item.icon_name = icon.clone();
                if let Some(icon) = icon {
                    widgets.root.set_icon_name(icon.as_str());
                }
                if let Some(icon) = self.item.icon_pixmap.clone() {
                    if let Some(icon) = icon.iter().max_by(|a, b| a.height.cmp(&b.height)) {
                        let pixels = icon.pixels.clone();
                        let vec = argb2rgba(pixels);
                        let bytes = glib::Bytes::from_owned(vec);
                        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
                            &bytes,
                            Colorspace::Rgb,
                            true,
                            8,
                            icon.width,
                            icon.height,
                            icon.width * 4,
                        );
                        let icon = gtk::Image::from_pixbuf(Some(&pixbuf));
                        widgets.root.set_child(Some(&icon))
                    }
                }
            }
            UpdateEvent::Menu(menu) => {
                self.menu = Some(menu);
                if let Some(action) = self.build_action_group() {
                    widgets.popover.insert_action_group("tray", Some(&action));
                }
                if let Some(menu) = self.build_model() {
                    widgets.popover.set_menu_model(Some(&menu));
                }
            }
            UpdateEvent::Title(title) => {
                self.item.title = title;
                widgets.root.set_tooltip_text(self.item.title.as_deref())
            }
            _ => {}
        };
    }
}

impl TrayItem {
    fn build_model(&self) -> Option<gio::Menu> {
        let value = self.menu.clone()?.submenus;
        self.build_menu(value)
    }

    fn build_menu(&self, value: Vec<MenuItem>) -> Option<gio::Menu> {
        let menu = gio::Menu::new();

        let sections = value
            .split(|it| it.menu_type == MenuType::Separator)
            .filter(|it| !it.is_empty())
            .collect::<Vec<&[MenuItem]>>();

        for section in sections {
            let section_menu = gio::Menu::new();
            for item in section {
                if !item.visible {
                    continue;
                }
                let item_clone = item.clone();
                if let Some(menu_item) = self.menu_item_for(item_clone) {
                    section_menu.append_item(&menu_item);
                }
            }
            menu.append_section(None, &section_menu);
        }

        Some(menu)
    }

    fn menu_item_for(&self, value: MenuItem) -> Option<gio::MenuItem> {
        if let MenuType::Separator = value.menu_type {
            panic!("Separators should get there!")
        }
        if !value.submenu.is_empty() {
            let menu = self.build_menu(value.submenu)?;
            return Some(gio::MenuItem::new_submenu(value.label.as_deref(), &menu));
        }

        let item = gio::MenuItem::new(
            value.label.as_deref(),
            Some(format!("{},{}", "tray", value.id).as_str()),
        );

        if let Some(icon_data) = value.icon_data {
            let bytes = glib::Bytes::from_owned(argb2rgba(icon_data));
            let icon = gio::BytesIcon::new(&bytes);
            item.set_icon(&icon);
        }
        if let Some(icon) = value.icon_name {
            match gio::Icon::for_string(icon.as_str()) {
                Ok(icon) => {
                    item.set_icon(&icon);
                }
                Err(err) => {
                    error!("Error setting icon for menu: {}", err)
                }
            }
        }
        let detailed_action: String = "tray.".to_owned() + value.id.to_string().as_str();
        item.set_detailed_action(detailed_action.as_str());

        Some(item)
    }

    fn build_action_group(&self) -> Option<gio::SimpleActionGroup> {
        let value = self.menu.clone()?;
        let action_group = gio::SimpleActionGroup::new();

        for item in value.submenus {
            for action in self.simple_action_for(item) {
                action_group.add_action(&action);
            }
        }

        Some(action_group)
    }

    fn simple_action_for(&self, value: MenuItem) -> Vec<gio::SimpleAction> {
        let mut vec = Vec::<gio::SimpleAction>::new();
        if value.menu_type == MenuType::Separator {
            return vec;
        }
        if !value.submenu.is_empty() {
            for child in value.submenu {
                let mut actions = self.simple_action_for(child);
                vec.append(actions.as_mut());
            }
        } else {
            let variant_value = match value.toggle_type {
                ToggleType::Checkmark | ToggleType::Radio => match value.toggle_state {
                    ToggleState::On => glib::Variant::from(true),
                    ToggleState::Off => glib::Variant::from(false),
                    ToggleState::Indeterminate => glib::Variant::from(-1),
                },
                ToggleType::CannotBeToggled => glib::Variant::from_none(glib::VariantTy::INT32),
            };
            let action = gio::SimpleAction::new_stateful(
                value.id.to_string().as_str(),
                None,
                &variant_value,
            );
            action.set_enabled(value.enabled);
            let menu_path = Box::new(self.item.menu.clone().unwrap());
            let key = Box::new(self.key.clone());
            let client = self.client.clone();
            action.connect_activate(move |_it, _state| {
                let rq = ActivateRequest {
                    address: key.as_ref().clone(),
                    menu_path: menu_path.as_ref().clone(),
                    submenu_id: value.id,
                };
                let client = client.clone();
                relm4::spawn_local(async move {
                    match client.activate(rq).await {
                        Ok(()) => {
                            debug!("Sent!")
                        }
                        Err(err) => {
                            debug!("{err}")
                        }
                    };
                });
            });
            vec.push(action);
        }

        vec
    }
}

fn argb2rgba(mut argb: Vec<u8>) -> Vec<u8> {
    for x in argb.chunks_exact_mut(4) {
        x.rotate_right(1);
    }
    argb
}
