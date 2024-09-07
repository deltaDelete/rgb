use gtk::prelude::{ButtonExt, WidgetExt};
use hyprland::dispatch;
use hyprland::dispatch::Dispatch;
use hyprland::dispatch::{DispatchType, WorkspaceIdentifierWithSpecial};
use log::error;
use relm4::prelude::FactoryComponent;
use relm4::{FactorySender, RelmWidgetExt};

#[derive(Debug, Clone)]
pub struct WorkspaceModel {
    pub id: hyprland::shared::WorkspaceId,
    pub name: String,
    pub monitor: String,
    pub monitor_id: hyprland::shared::MonitorId,
    pub windows: u16,
    pub fullscreen: bool,
    pub last_window: hyprland::shared::Address,
    pub last_window_title: String,
    is_active: bool,
}

impl Into<hyprland::data::Workspace> for WorkspaceModel {
    fn into(self) -> hyprland::data::Workspace {
        hyprland::data::Workspace {
            id: self.id.clone(),
            name: self.name.clone(),
            monitor: self.monitor.clone(),
            monitor_id: self.monitor_id.clone(),
            windows: self.windows.clone(),
            fullscreen: self.fullscreen.clone(),
            last_window: self.last_window.clone(),
            last_window_title: self.last_window_title.clone(),
        }
    }
}

impl From<hyprland::data::Workspace> for WorkspaceModel {
    fn from(value: hyprland::data::Workspace) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            monitor: value.monitor.clone(),
            monitor_id: value.monitor_id.clone(),
            windows: value.windows.clone(),
            fullscreen: value.fullscreen.clone(),
            last_window: value.last_window.clone(),
            last_window_title: value.last_window_title.clone(),
            is_active: false,
        }
    }
}

#[derive(Debug)]
pub enum WorkspaceMessage {
    SwitchTo,
    ActiveIdChanged { id: hyprland::shared::WorkspaceId },
}

#[relm4::factory(pub)]
impl FactoryComponent for WorkspaceModel {
    type Input = WorkspaceMessage;
    type Output = ();
    type Init = hyprland::data::Workspace;
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_label: self.id.to_string().as_str(),
            // #[watch]
            // set_css_classes: if self.is_active { &["workspaces__workspace", "active"] } else { &["workspaces__workspace"] },
            add_css_class: "workspaces__workspace",
            #[watch]
            set_class_active: ("active", self.is_active),
            connect_clicked => WorkspaceMessage::SwitchTo
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        init.into()
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            WorkspaceMessage::SwitchTo => {
                if let Err(e) = dispatch!(Workspace,WorkspaceIdentifierWithSpecial::Id(self.id)) {
                    error!("{e}");
                };
            }
            WorkspaceMessage::ActiveIdChanged { id } => {
                self.is_active = self.id == id;
            }
        }
    }
}
