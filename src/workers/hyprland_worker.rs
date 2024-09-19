use crate::ext::ById;
use crate::widgets::HyprlandMessage;
use crate::widgets::HyprlandMessage::{Active, Add, Remove};
use hyprland::data::Workspace;
use hyprland::event_listener::{
    LayoutEvent, MonitorEventData, WindowEventData, WorkspaceDestroyedEventData,
};
use hyprland::shared::{WorkspaceId, WorkspaceType};
use log::{debug, error};
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
                    Self::on_workspace_changed(&sender, it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_workspace_added_handler(move |it| {
                    Self::on_workspace_added(&sender, it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_workspace_destroy_handler(move |it| {
                    Self::on_workspace_destroyed(&sender, it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_active_monitor_change_handler(move |it| {
                    Self::on_active_monitor_changed(&sender, it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_active_window_change_handler(move |it| {
                    if it.is_none() {
                        error!("Got empty activewindow event expected some!");
                    }
                    Self::on_active_window_changed(&sender, it);
                });
            }
            {
                let sender = sender.clone();
                listener.add_monitor_removed_handler(move |it| {
                    if Self::on_monitor_removed(&sender, it).is_none() {
                        error!("Failed to send monitor added event!");
                    }
                });
            }
            {
                let sender = sender.clone();
                listener.add_monitor_added_handler(move |it| {
                    if Self::on_monitor_added(&sender, it).is_none() {
                        error!("Failed to send monitor added event!");
                    }
                });
            }
            {
                let sender = sender.clone();
                listener.add_keyboard_layout_change_handler(move |it| {
                    if Self::on_keyboard_layout_changed(&sender, it).is_none() {
                        error!("Failed to send keyboard layout event!");
                    }
                });
            }
            listener.start_listener().expect("");
        });
    }

    fn on_active_monitor_changed(
        sender: &ComponentSender<HyprlandHandler>,
        event_data: MonitorEventData,
    ) -> Option<()> {
        debug!("Active monitor changed: {event_data:?}");
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
        sender: &ComponentSender<HyprlandHandler>,
        event_data: WorkspaceDestroyedEventData,
    ) -> Option<()> {
        debug!("Workspace destroyed: {event_data:?}");
        sender
            .output(Remove {
                id: event_data.workspace_id,
            })
            .ok()?;
        Some(())
    }

    fn on_workspace_changed(
        sender: &ComponentSender<HyprlandHandler>,
        workspace_type: WorkspaceType,
    ) -> Option<()> {
        debug!("Workspace changed: {workspace_type:?}");
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
        sender: &ComponentSender<HyprlandHandler>,
        workspace_type: WorkspaceType,
    ) -> Option<()> {
        debug!("Workspace added: {workspace_type:?}");
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
        sender: &ComponentSender<HyprlandHandler>,
        event_data: Option<WindowEventData>,
    ) -> Option<()> {
        debug!("Active window changed: {event_data:?}");
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
        sender: &ComponentSender<HyprlandHandler>,
        event_data: LayoutEvent,
    ) -> Option<()> {
        debug!("Keyboard layout changed: {event_data:?}");
        sender.output(event_data.into()).ok()?;
        
        Some(())
    }

    fn on_monitor_added(sender: &ComponentSender<HyprlandHandler>, monitor: String) -> Option<()> {
        debug!("Monitor added: {monitor}");
        sender.output(HyprlandMessage::AddMonitor { monitor }).ok()?;

        Some(())
    }

    fn on_monitor_removed(sender: &ComponentSender<HyprlandHandler>, monitor: String) -> Option<()> {
        debug!("Monitor removed {monitor}");
        sender.output(HyprlandMessage::RemoveMonitor { monitor }).ok()?;

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
