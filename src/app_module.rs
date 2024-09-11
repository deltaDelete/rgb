use crate::app_module::AppMessage::UpdateCss;
use crate::widgets::{
    DateTime, Focused, HyprlandMessage, Language, PowerMenu, SysTray, WorkspacesModel,
};
use crate::workers::HyprlandHandler;
use gtk::gio;
use gtk::prelude::WidgetExt;
use gtk::prelude::{BoxExt, GtkWindowExt, OrientableExt};
use gtk::{gdk, StyleContext};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use log::error;
use relm4::prelude::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController,
    SimpleAsyncComponent,
};
use relm4::AsyncComponentSender;
use relm4::Component;
use relm4::ComponentController;
use relm4::Controller;
use relm4::RelmWidgetExt;
use relm4::WorkerController;
use std::path::Path;

pub struct AppModel {
    workspaces: Controller<WorkspacesModel>,
    focused: Controller<Focused>,
    #[allow(dead_code)]
    handler: WorkerController<HyprlandHandler>,
    css_related: CssRelated,
    power_menu: Controller<PowerMenu>,
    sys_tray: AsyncController<SysTray>,
    datetime: AsyncController<DateTime>,
    language: AsyncController<Language>,
}

struct CssRelated {
    css_provider: gtk::CssProvider,
    file: gio::File,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppMessage {
    UpdateCss,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for AppModel {
    type Input = AppMessage;
    type Output = ();
    type Init = ();

    view! {
        gtk::Window {
            set_title: Some("Fancy title!"),
            set_decorated: true,

            gtk::CenterBox {
                set_expand: true,
                set_orientation: gtk::Orientation::Horizontal,

                #[wrap(Some)]
                set_start_widget: start = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    model.focused.widget(),
                },
                #[wrap(Some)]
                set_center_widget: center = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    model.workspaces.widget(),
                },
                #[wrap(Some)]
                set_end_widget: end = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    model.language.widget(),
                    model.sys_tray.widget(),
                    model.datetime.widget(),
                    model.power_menu.widget(),
                }
            }
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let workspaces = WorkspacesModel::builder().launch(()).detach();
        let focused = Focused::builder().launch(()).detach();
        let power_menu = PowerMenu::builder().launch(()).detach();
        let sys_tray = SysTray::builder().launch(()).detach();
        let datetime = DateTime::builder().launch(()).detach();
        let language = Language::builder().launch(()).detach();

        let language_sender = language.sender().clone();
        let workspaces_sender = workspaces.sender().clone();
        let focused_sender = focused.sender().clone();
        let handler: WorkerController<HyprlandHandler> = HyprlandHandler::builder()
            .detach_worker(())
            .forward(&workspaces_sender, move |message| {
                let message_clone = message.clone();
                match message_clone {
                    HyprlandMessage::ActiveWindow { .. } => {
                        if let Err(e) = focused_sender.send(message_clone) {
                            error!("Error sending message to Focused: {:?}", e);
                        }
                    }
                    HyprlandMessage::SwitchKeyboardLayout { .. } => {
                        if let Err(e) = language_sender.send(message_clone) {
                            error!("Error sending message to Language: {:?}", e);
                        }
                    }
                    _ => {}
                };
                message
            });

        let css_related = {
            let path = Path::new("./res/style.css");
            let css_provider = gtk::CssProvider::new();
            let file = gio::File::for_path(path);
            css_provider.load_from_file(&file);
            #[allow(deprecated)]
            StyleContext::add_provider_for_display(
                &gdk::Display::default().unwrap(),
                &css_provider,
                800,
            );

            CssRelated { css_provider, file }
        };

        let model = Self {
            workspaces, // popover: None,
            focused,
            handler,
            css_related,
            power_menu,
            sys_tray,
            datetime,
            language,
        };

        let widgets = view_output!();

        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Right, true);
        root.set_namespace("regbar");
        root.set_height_request(50);
        root.set_margin(Edge::Left, 20);
        root.set_margin(Edge::Right, 20);
        root.set_margin(Edge::Top, 12);
        root.set_margin(Edge::Bottom, 12);
        root.set_keyboard_mode(KeyboardMode::OnDemand);
        // root.set_monitor_by_connector("DP-1");
        root.auto_exclusive_zone_enable();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            UpdateCss => {
                self.css_related
                    .css_provider
                    .load_from_file(&self.css_related.file);
            }
        }
    }
}
