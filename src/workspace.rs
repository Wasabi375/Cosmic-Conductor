use crate::{
    args::WorkspaceIdent,
    cosmic::AppData,
    output::{self, print_displays},
    print_otpion,
};

use cosmic_client_toolkit::{
    toplevel_info::ToplevelInfo,
    workspace::{Workspace, WorkspaceGroup},
};
use cosmic_protocols::workspace::v2::client::zcosmic_workspace_handle_v2::{
    TilingState, WorkspaceCapabilities,
};
use log::{error, warn};
use wayland_client::Proxy;
use wayland_protocols::ext::workspace::v1::client::ext_workspace_group_handle_v1::GroupCapabilities;
use wayland_protocols::ext::workspace::v1::client::ext_workspace_handle_v1::WorkspaceCapabilities as ExtWorkspaceCapabilities;

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

pub fn list(app_data: &AppData, print_capabilities: bool) {
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
        if print_capabilities {
            println!("Capabilities:");
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Move)
            {
                println!("\tmove");
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Pin)
            {
                println!("\tpin");
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Rename)
            {
                println!("\trename");
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Pin)
            {
                println!("\tset tiling");
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Activate)
            {
                println!("\tactivate");
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Assign)
            {
                println!("\tassign");
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Deactivate)
            {
                println!("\tdeactivate");
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Remove)
            {
                println!("\tremove");
            }
        }

        println!();
    }
}

pub fn get_workspace<'a>(
    app_data: &'a AppData,
    workspace: &WorkspaceIdent,
) -> Option<(&'a WorkspaceGroup, usize, &'a Workspace)> {
    if let Some(display) = workspace.display.as_ref() {
        let Some(group) = app_data.workspace_state.workspace_groups().find(|group| {
            group
                .outputs
                .iter()
                .filter_map(|o| app_data.output_state.info(o))
                .any(|o| &output::display_name(&o) == display)
        }) else {
            error!("Unknonw display: {}", display);
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
            .find(|(_, w)| w.name == workspace.name)
        else {
            error!(
                "Workspace {} does not exist on display {display}",
                workspace.name
            );
            return None;
        };

        Some((group, workspace_pos, workspace))
    } else {
        let mut candidate_workspaces = app_data
            .workspace_state
            .workspaces()
            .filter(|w| w.name == workspace.name);

        let Some(workspace) = candidate_workspaces.next() else {
            error!("Workspace {} does not exist", workspace.name);
            return None;
        };

        if candidate_workspaces.next().is_some() {
            error!(
                "Found multiple workspaces with name {}. Specify display to narrow down selection",
                workspace.name
            );
            return None;
        }

        let Some((workspace_pos, group)) = app_data
            .workspace_state
            .workspace_groups()
            .enumerate()
            .find(|(_, group)| group.workspaces.contains(&workspace.handle))
        else {
            error!(
                "Found workspace {} but could not access it's group",
                workspace.name
            );
            return None;
        };

        Some((group, workspace_pos, workspace))
    }
}

/// Move the workspace to position and display
///
/// # Arguments
///
/// * `target_position` is counted starting at 1.
/// * if the `target_display` is `None` the workspace is moved within it's worksapce group
pub fn move_to(
    app_data: &AppData,
    workspace: WorkspaceIdent,
    target_position: usize,
    target_display: Option<&str>,
) {
    let Ok(workspace_manager) = app_data.workspace_state.workspace_manager().get() else {
        warn!("could not get acccess to workspace manager");
        return;
    };

    let Some((orig_group, current_pos, workspace)) = get_workspace(app_data, &workspace) else {
        return;
    };

    let group = if let Some(target_display) = target_display {
        let Some(group) = app_data.workspace_state.workspace_groups().find(|group| {
            group
                .outputs
                .iter()
                .filter_map(|o| app_data.output_state.info(o))
                .any(|o| &output::display_name(&o) == target_display)
        }) else {
            error!("Unknonw display: {}", target_display);
            return;
        };
        group
    } else {
        orig_group
    };

    let position = if target_position == 0 {
        warn!("position should never be 0. Workspaces are counted starting at 1");
        0
    } else {
        target_position - 1
    };

    let position = if position >= group.workspaces.len() {
        let real_pos = group.workspaces.len() - 1;
        if position != usize::MAX - 1 {
            warn!("{position} to large. Workspace will be moved to the end at {real_pos}");
        }
        real_pos
    } else {
        position
    };

    if current_pos == position && orig_group.handle.id() == group.handle.id() {
        warn!(
            "workspace {} already at position {position}",
            workspace.name
        );
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
        error!(
            "INTERNAL: No cosmic handle for workspace {}",
            workspace.name
        );
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
