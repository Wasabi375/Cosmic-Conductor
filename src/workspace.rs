use crate::{cosmic::AppData, output::print_displays, print_otpion};

use cosmic_client_toolkit::{
    toplevel_info::ToplevelInfo,
    workspace::{Workspace, WorkspaceGroup},
};
use cosmic_protocols::workspace::v2::client::zcosmic_workspace_handle_v2::TilingState;
use wayland_protocols::ext::workspace::v1::client::ext_workspace_group_handle_v1::GroupCapabilities;

pub fn list_groups(app_data: &AppData) {
    println!("Workspace Groups:");
    for wg in app_data.workspace_state.workspace_groups() {
        print_displays(app_data, &wg.outputs);
        println!("workspace count: {}", wg.workspaces.len());
        println!(
            "can create workspace: {}",
            wg.capabilities.contains(GroupCapabilities::CreateWorkspace)
        );
        println!();
    }
}

pub fn list(app_data: &AppData) {
    let _ = app_data;
    println!("Workspaces:");
    for workspace in app_data.workspace_state.workspaces() {
        println!("Name: {}", workspace.name);
        print_otpion(workspace.id.as_ref(), "id");
        let displays =
            get_groups_for_workspace(workspace, app_data).flat_map(|wg| wg.outputs.iter());
        print_displays(app_data, displays);
        println!("Tiling: {}", is_workspace_tiling(workspace));
        println!(
            "Toplevel count: {}",
            workspace_toplevels(workspace, app_data).count()
        );
        println!();
    }
}

pub fn move_to(workspace_id: String, position: u8) {
    todo!()
}

pub fn is_workspace_tiling(workspace: &Workspace) -> bool {
    match workspace.tiling.map(|s| s.into_result().ok()).flatten() {
        Some(TilingState::TilingEnabled) => true,
        Some(_) | None => false,
    }
}

pub fn get_groups_for_workspace<'a>(
    workspace: &Workspace,
    app_data: &'a AppData,
) -> impl Iterator<Item = &'a WorkspaceGroup> {
    app_data
        .workspace_state
        .workspace_groups()
        .filter(|wg| wg.workspaces.contains(&workspace.handle))
}

pub fn workspace_toplevels<'a>(
    workspace: &Workspace,
    app_data: &'a AppData,
) -> impl Iterator<Item = &'a ToplevelInfo> {
    app_data
        .toplevel_info_state
        .toplevels()
        .filter(|t| t.workspace.contains(&workspace.handle))
}
