use crate::{
    cosmic::AppData,
    output::{self, print_displays},
    print_otpion,
};

use cosmic_client_toolkit::{
    toplevel_info::ToplevelInfo,
    workspace::{Workspace, WorkspaceGroup},
};
use cosmic_protocols::workspace::v2::client::zcosmic_workspace_handle_v2::TilingState;
use log::{error, warn};
use wayland_client::Proxy;
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
        print_otpion(workspace.id.as_ref(), "wayland id");
        log::debug!(
            "CosmicId: {:?}",
            workspace.cosmic_handle.as_ref().map(|h| h.id())
        );
        log::debug!("ExtId: {:?}", workspace.handle.id());
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

pub fn get_workspace<'a>(
    app_data: &'a AppData,
    workspace: &str,
    display: &str,
) -> Option<(&'a WorkspaceGroup, usize, &'a Workspace)> {
    let Some(group) = app_data.workspace_state.workspace_groups().find(|group| {
        group
            .outputs
            .iter()
            .filter_map(|o| app_data.output_state.info(o))
            .any(|o| output::display_name(&o) == display)
    }) else {
        error!("Unknonw display: {display}");
        return None;
    };

    let Some((workspace_pos, workspace)) = group
        .workspaces
        .iter()
        .enumerate()
        .filter_map(|(i, handle)| {
            log::debug!("{i}: {handle:?}");
            app_data
                .workspace_state
                .workspace_info(handle)
                .map(|info| (i, info))
        })
        .find(|(_, w)| w.name == workspace)
    else {
        error!("Workspace {workspace} does not exist on display {display}");
        return None;
    };

    Some((group, workspace_pos, workspace))
}

pub fn move_to(app_data: &AppData, workspace_name: String, display: String, position: usize) {
    let Ok(workspace_manager) = app_data.workspace_state.workspace_manager().get() else {
        warn!("could not get acccess to workspace manager");
        return;
    };

    let Some((group, current_pos, workspace)) = get_workspace(app_data, &workspace_name, &display)
    else {
        return;
    };

    let position = if position >= dbg!(group.workspaces.len()) {
        let real_pos = group.workspaces.len() - 1;
        warn!("{position} to large. Workspace will be moved to the end at {real_pos}");
        real_pos
    } else {
        position
    };

    if dbg!(current_pos) == position {
        warn!("workspace {workspace_name} already at position {position}");
        return;
    }

    let (other_pos, move_after) = if position == 0 {
        (1, false)
    } else {
        (position - 1, true)
    };
    let other_workspace = group.workspaces.iter().nth(other_pos).expect(
        "other pos is valid, because we move after other and position is valid,
        unless position is 0 in which case 1 is valid, because current_pos > 0",
    );

    // aparently every value but 0 is ignored. Not sure what this means, but the current
    // cosmic-compositor just checks that it is 0. Events with other values are ignored.
    const AXIS: u32 = 0;

    let Some(cosmic_handle) = workspace.cosmic_handle.as_ref() else {
        error!("INTERNAL: No cosmic handle for workspace {workspace_name}");
        return;
    };

    if move_after {
        log::debug!(
            "move {:?} after {:?}",
            cosmic_handle.id(),
            other_workspace.id()
        );
        cosmic_handle.move_after(other_workspace, AXIS);
    } else {
        log::debug!(
            "move {:?} before {:?}",
            cosmic_handle.id(),
            other_workspace.id()
        );
        cosmic_handle.move_before(other_workspace, AXIS);
    }
    workspace_manager.commit();
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
