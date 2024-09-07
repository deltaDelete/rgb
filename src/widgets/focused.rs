use crate::widgets::WorkspacesMessage;
use hyprland::prelude::HyprDataActiveOptional;
use hyprland::shared::Address;
use gtk::gdk::Display;
use gtk::prelude::{OrientableExt, WidgetExt};
use gtk::Orientation;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use std::ops::Not;

pub struct Focused {
    title: String,
    icon_name: String,
    address: Address,
    show: bool,
    has_icon: bool,
    has_title: bool,
    icon_theme: gtk::IconTheme,
}

impl Default for Focused {
    fn default() -> Self {
        Self {
            title: String::from(""),
            icon_name: String::from(""),
            address: Address::new(""),
            show: false,
            has_title: false,
            has_icon: false,
            icon_theme: gtk::IconTheme::for_display(&Display::default().unwrap())
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Focused {
    type Input = WorkspacesMessage;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Revealer {
            set_transition_type: gtk::RevealerTransitionType::Crossfade,
            set_transition_duration: 200,
            #[watch]
            set_reveal_child: model.show,
            #[watch]
            set_tooltip_text: Some(model.address.to_string().as_str()),
            gtk::Box {
                set_size_group: &gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal),
                set_orientation: Orientation::Horizontal,
                add_css_class: "module",
                add_css_class: "focused",

                #[name = "icon"]
                gtk::Image {
                    #[watch]
                    set_icon_name: Some(model.icon_name.as_str()),
                    #[watch]
                    set_visible: model.icon_theme.has_icon(model.icon_name.as_str()),
                    set_use_fallback: true,
                    set_pixel_size: 24,
                    add_css_class: "focused__icon",
                },

                #[name = "title"]
                gtk::Label {
                    #[watch]
                    set_label: model.title.as_str(),
                    #[watch]
                    set_visible: model.title.is_empty().not(),
                    set_valign: gtk::Align::Center,
                    add_css_class: "focused__title",
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::default();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) -> () {
        match message {
            WorkspacesMessage::ActiveWindow { window } => {
                if let Some(window) = window {
                    self.title = window.window_title.trim().into();
                    self.address = window.window_address;
                    self.icon_name = window.window_class;
                } else {
                    let result = hyprland::data::Client::get_active()
                        .expect("Expected to get active")
                        .expect("Active was None");
                    self.title = result.title.trim().into();
                    self.icon_name = result.class;
                    self.address = result.address;
                }
                self.has_icon = self.icon_theme.has_icon(self.icon_name.as_str());
                self.has_title = self.title.trim().is_empty().not();
                self.show = self.has_title || self.has_icon;
            }
            _ => {}
        };
    }
}
