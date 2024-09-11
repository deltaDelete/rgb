use hyprland::data::Workspace;
use hyprland::prelude::HyprData;
use hyprland::shared::WorkspaceId;

pub trait ById<T> {
    fn get_by_id(id: T) -> Self;
}

impl ById<WorkspaceId> for Option<Workspace> {
    fn get_by_id(id: i32) -> Option<Workspace> {
        let workspaces = hyprland::data::Workspaces::get().ok()?;
        let mut iter = workspaces.iter();
        iter.find(|&it| it.id == id).cloned()
    }
}
