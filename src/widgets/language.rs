use crate::widgets::HyprlandMessage;
use gtk::prelude::{ButtonExt, WidgetExt};
use log::error;
use relm4::component::AsyncComponentParts;
use relm4::component::SimpleAsyncComponent;
use relm4::AsyncComponentSender;

#[derive(Debug, Clone)]
pub struct Language {
    device: String,
    layout: String,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for Language {
    type Init = ();
    type Input = HyprlandMessage;
    type Output = ();

    view! {
        #[root]
        gtk::Button {
            add_css_class: "module",
            add_css_class: "hoverable",
            add_css_class: "language",
            #[watch]
            set_label: model.layout.as_str(),
            connect_clicked[model] => move |_btn| {
                if let Err(e) = hyprland::ctl::switch_xkb_layout::call(model.device.clone(), hyprland::ctl::switch_xkb_layout::SwitchXKBLayoutCmdTypes::Next) {
                    error!("Error switching keyboard layout: {e}");
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            device: String::new(),
            layout: String::new(),
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        if let HyprlandMessage::SwitchKeyboardLayout {
            keyboard_name,
            layout_name,
        } = message
        {
            if layout_name.is_empty() {
                let split = keyboard_name.split(',').collect::<Vec<&str>>();
                if let Some(first) = split.first() {
                    self.device = first.to_string();
                }
                if let Some(last) = split.last() {
                    self.layout = last.to_string();
                }
            } else {
                self.layout = layout_name;
            }
        }
    }
}
