use crate::ext::ById;
use crate::widgets::HyprlandMessage;
use crate::widgets::HyprlandMessage::{Active, Add, Remove};
use hyprland::data::Workspace;
use hyprland::event_listener::{
    LayoutEvent, MonitorEventData, WindowEventData, WorkspaceDestroyedEventData,
};
use hyprland::shared::{WorkspaceId, WorkspaceType};
use log::{error, info};
use relm4::{ComponentSender, Worker};

pub struct HyprlandHandler;

impl Worker for HyprlandHandler {
    type Init = ();
    type Input = ();
    type Output = HyprlandMessage;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        _sender.input(());
        Self {}
    }

    fn update(&mut self, _msg: (), sender: ComponentSender<Self>) {
        self.start(sender)
    }
}

impl HyprlandHandler {
    fn start(&mut self, _sender: ComponentSender<Self>) {
        let sender = _sender.clone();
        relm4::spawn_blocking(move || {
            let mut listener = hyprland::event_listener::EventListener::new();
            {
                let sender = sender.clone();
                listener.add_workspace_change_handler(move |it| {
                    Self::on_workspace_change(sender.clone(), it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_workspace_added_handler(move |it| {
                    Self::on_workspace_added(sender.clone(), it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_workspace_destroy_handler(move |it| {
                    Self::on_workspace_destroyed(sender.clone(), it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_active_monitor_change_handler(move |it| {
                    Self::on_active_monitor_changed(sender.clone(), it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_active_window_change_handler(move |it| {
                    if it.is_none() {
                        error!("Got empty activewindow event expected some!");
                    }
                    Self::on_active_window_changed(sender.clone(), it);
                });
            }
            {
                listener.add_monitor_removed_handler(|it| {
                    info!("Monitor removed. {}", it);
                });
                listener.add_monitor_added_handler(|it| {
                    info!("Monitor added. {}", it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_keyboard_layout_change_handler(move |it| {
                    if Self::on_keyboard_layout_changed(sender.clone(), it).is_none() {
                        error!("Failed to send keyboard layout event!");
                    }
                });
            }
            listener.start_listener().expect("");
        });
    }

    fn on_active_monitor_changed(
        sender: ComponentSender<HyprlandHandler>,
        event_data: MonitorEventData,
    ) -> Option<()> {
        if let WorkspaceType::Regular(regular) = event_data.workspace {
            let id = regular
                .parse::<WorkspaceId>()
                .map_err(|it| {
                    error!("{}", it);
                    it
                })
                .ok()?;
            sender.output(Active { id }).ok()?;
        }
        Some(())
    }

    fn on_workspace_destroyed(
        sender: ComponentSender<HyprlandHandler>,
        event_data: WorkspaceDestroyedEventData,
    ) -> Option<()> {
        sender
            .output(Remove {
                id: event_data.workspace_id,
            })
            .ok()?;
        Some(())
    }

    fn on_workspace_change(
        sender: ComponentSender<HyprlandHandler>,
        workspace_type: WorkspaceType,
    ) -> Option<()> {
        if let WorkspaceType::Regular(regular) = workspace_type {
            let id = regular
                .parse::<WorkspaceId>()
                .map_err(|it| {
                    error!("{}", it);
                    it
                })
                .ok()?;
            sender.output(Active { id }).ok()?;
        }
        Some(())
    }

    fn on_workspace_added(
        sender: ComponentSender<HyprlandHandler>,
        workspace_type: WorkspaceType,
    ) -> Option<()> {
        if let WorkspaceType::Regular(regular) = workspace_type {
            let id = regular
                .parse::<WorkspaceId>()
                .map_err(|it| {
                    error!("{}", it);
                    it
                })
                .ok()?;
            sender
                .output(Add {
                    workspace: Option::<Workspace>::get_by_id(id)?,
                })
                .ok()?;
        }
        Some(())
    }

    fn on_active_window_changed(
        sender: ComponentSender<HyprlandHandler>,
        event_data: Option<WindowEventData>,
    ) -> Option<()> {
        sender
            .output(HyprlandMessage::ActiveWindow {
                // Because hyprland-rs makes activewindow event to have None value
                // when either title or class is empty, using this workaround
                window: event_data,
            })
            .ok()?;
        Some(())
    }

    fn on_keyboard_layout_changed(
        sender: ComponentSender<HyprlandHandler>,
        event_data: LayoutEvent,
    ) -> Option<()> {
        sender.output(event_data.into()).ok()?;
        Some(())
    }
}

impl From<LayoutEvent> for HyprlandMessage {
    fn from(value: LayoutEvent) -> Self {
        Self::SwitchKeyboardLayout {
            keyboard_name: value.keyboard_name,
            layout_name: value.layout_name,
        }
    }
}

