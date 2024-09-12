use crate::ext::{ForName, KeyboardLayoutEx};
use crate::widgets::HyprlandMessage;
use gtk::prelude::{ButtonExt, WidgetExt};
use hyprland::data::Keyboard;
use hyprland::shared::HyprData;
use log::{error, info};
use relm4::component::AsyncComponentParts;
use relm4::component::SimpleAsyncComponent;
use relm4::{AsyncComponentSender, RelmWidgetExt};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct Language {
    device: Arc<Mutex<Option<hyprland::data::Keyboard>>>,
    layout: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageInit {
    pub device_name: String,
}

#[allow(dead_code)]
impl LanguageInit {
    pub fn new(device_name: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
        }
    }

    pub fn new_for_string(device_name: String) -> Self {
        Self { device_name }
    }
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for Language {
    type Input = HyprlandMessage;
    type Output = ();
    type Init = LanguageInit;

    view! {
        #[root]
        gtk::Button {
            add_css_class: "module",
            add_css_class: "hoverable",
            add_css_class: "language",
            #[watch]
            set_label: &model.layout().unwrap_or_else(|| "NULL".to_string()).to_uppercase(),
            // #[watch]
            // set_tooltip_text: model.device().map(|it| it.name).as_deref(),
            connect_clicked => move |_btn| {
                // TODO find why freezes
                relm4::spawn_blocking(|| {
                    if let Err(e) = hyprland::ctl::switch_xkb_layout::call("all", hyprland::ctl::switch_xkb_layout::SwitchXKBLayoutCmdTypes::Next) {
                        error!("Error switching keyboard layout: {e}");
                    };
                });
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            device: Arc::new(Mutex::new(None)),
            layout: Arc::new(Mutex::new(None)),
        };
        model.set_device_async(_init.device_name).await;
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        if let HyprlandMessage::SwitchKeyboardLayout {
            keyboard_name,
            layout_name,
        } = message
        {
            let (keyboard_name, layout_name): (String, String) = {
                if layout_name.is_empty() {
                    let split = keyboard_name.split(',').collect::<Vec<&str>>();
                    (split[0].to_string(), split[1].to_string())
                } else {
                    (keyboard_name, layout_name)
                }
            };

            if self.device().is_some_and(|it| keyboard_name == it.name) {
                self.set_layout(
                    xkb_data::KeyboardLayout::find(&layout_name).and_then(|it| it.localized()),
                );
            }
        }
    }
}

impl Language {
    async fn set_device_async(&self, device_name: String) -> Option<()> {
        let devices = hyprland::data::Devices::get_async().await.ok()?;
        let keyboard = devices.keyboards.iter().find(|it| it.name == device_name);
        if let Ok(mut device) = self.device.lock() {
            *device = keyboard.cloned();
            let layout = keyboard.cloned().and_then(|it| {
                xkb_data::KeyboardLayout::find(&it.active_keymap)
            }).and_then(|it| it.localized());
            self.set_layout(layout);
        }
        Some(())
    }

    fn set_layout(&self, value: Option<String>) -> Option<()> {
        let mut layout = self.layout.lock().ok()?;
        *layout = value;
        Some(())
    }

    fn device(&self) -> Option<Keyboard> {
        self.device.lock().unwrap().clone()
    }

    fn layout(&self) -> Option<String> {
        self.layout.try_lock().ok()?.clone()
    }
}

impl ForName for Keyboard {
    fn for_name(name: &str) -> Option<Keyboard> {
        let devices = hyprland::data::Devices::get().ok()?;
        let keyboard = devices.keyboards.iter().find(|it| it.name == name)?;
        Some(keyboard.clone())
    }

    async fn for_name_async(name: &str) -> Option<Self> {
        let devices = hyprland::data::Devices::get_async().await.ok()?;
        let keyboard = devices.keyboards.iter().find(|it| it.name == name)?;
        Some(keyboard.clone())
    }
}
