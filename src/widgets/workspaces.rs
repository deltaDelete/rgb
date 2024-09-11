use crate::widgets::{WorkspaceMessage, WorkspaceModel};
use gtk::prelude::OrientableExt;
use gtk::prelude::WidgetExt;
use gtk::EventControllerScrollFlags;
use hyprland::data::Workspace as WorkspaceStruct;
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::event_listener::WindowEventData;
use hyprland::prelude::{HyprData, HyprDataActive, HyprDataVec};
use log::error;
use relm4::factory::FactoryVecDeque;
use relm4::gtk::glib::Propagation;
use relm4::RelmWidgetExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::ops::Neg;

pub struct WorkspacesModel {
    workspaces: FactoryVecDeque<WorkspaceModel>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HyprlandMessage {
    Add {
        workspace: WorkspaceStruct,
    },
    Remove {
        id: hyprland::shared::WorkspaceId,
    },
    Active {
        id: hyprland::shared::WorkspaceId,
    },
    Start,
    ActiveWindow {
        window: Option<WindowEventData>,
    },
    SwitchKeyboardLayout {
        keyboard_name: String,
        layout_name: String,
    },
}

#[relm4::component(pub)]
impl SimpleComponent for WorkspacesModel {
    type Input = HyprlandMessage;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "module",
            add_css_class: "workspaces",
            add_controller = gtk::EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES) {
                connect_scroll => |_, x, y| {
                    let value = x + y;
                    if value < 0f64 {
                        if let Err(e) = hyprland::dispatch!(Workspace,WorkspaceIdentifierWithSpecial::Relative(1)) {
                            error!("{e}");
                        };
                        Propagation::Proceed
                    } else if value > 0f64 {
                        if let Err(e) = hyprland::dispatch!(Workspace,WorkspaceIdentifierWithSpecial::Relative(1.neg())) {
                            error!("{e}");
                        };
                        Propagation::Proceed
                    } else {
                        Propagation::Stop
                    }

                }
            },
            #[local_ref]
            workspaces_box -> gtk::Box {
                add_css_class: "workspaces__box",
                set_size_group: &gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
                set_orientation: gtk::Orientation::Horizontal,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let workspaces = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = WorkspacesModel { workspaces };

        let workspaces_box = model.workspaces.widget();
        let widgets = view_output!();

        Self::init_workspaces(sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            HyprlandMessage::Add { workspace } => {
                let mut guard = self.workspaces.guard();
                let some_index = guard.iter().rposition(|it| it.id < workspace.id);
                if let Some(index) = some_index {
                    guard.insert(index + 1, workspace);
                } else {
                    guard.push_front(workspace);
                }
                guard.drop();
            }
            HyprlandMessage::Remove { id } => {
                let mut guard = self.workspaces.guard();
                let some_index = guard.iter().rposition(|it| it.id == id);
                if let Some(index) = some_index {
                    guard.remove(index);
                }
                guard.drop();
            }
            HyprlandMessage::Active { id } => {
                let guard = self.workspaces.guard();
                for (index, _item) in guard.iter().enumerate() {
                    guard.send(index, WorkspaceMessage::ActiveIdChanged { id })
                }
            }
            _ => {}
        }
    }
}

impl WorkspacesModel {
    fn init_workspaces(sender: ComponentSender<WorkspacesModel>) {
        let mut ws = hyprland::data::Workspaces::get().unwrap().to_vec();
        ws.sort_by(|a, b| a.id.cmp(&b.id));
        ws.iter().for_each(|it| {
            sender.input(HyprlandMessage::Add {
                workspace: it.clone(),
            });
        });
        if let Ok(active_workspace) = hyprland::data::Workspace::get_active() {
            sender.input(HyprlandMessage::Active {
                id: active_workspace.id,
            });
        }
    }
}
