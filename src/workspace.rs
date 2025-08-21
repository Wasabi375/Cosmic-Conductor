use std::io::Write;

use crate::{
    args::WorkspaceIdent,
    cosmic::AppData,
    output::{self, print_displays},
    print::{ListOptions, Print, PrintList},
};

use anyhow::{Context, Result, bail};
use cosmic_client_toolkit::{
    toplevel_info::ToplevelInfo,
    workspace::{Workspace, WorkspaceGroup},
};
use cosmic_protocols::workspace::v2::client::zcosmic_workspace_handle_v2::{
    TilingState, WorkspaceCapabilities,
};
use log::warn;
use wayland_client::Proxy;
use wayland_protocols::ext::workspace::v1::client::ext_workspace_handle_v1::WorkspaceCapabilities as ExtWorkspaceCapabilities;
use wayland_protocols::ext::workspace::v1::client::{
    ext_workspace_group_handle_v1::GroupCapabilities, ext_workspace_handle_v1,
};

pub fn list_groups<W: Write>(app_data: &AppData, printer: &mut impl Print<W>) -> Result<()> {
    let mut printer = printer.sub_list("Workspace Groups")?;
    for wg in app_data.workspace_state.workspace_groups() {
        let mut printer = printer.sub_struct()?;
        print_displays(app_data, &mut printer, &wg.outputs)?;
        printer.field("workspace count", wg.workspaces.len())?;
        printer.field(
            "can create workspace",
            wg.capabilities.contains(GroupCapabilities::CreateWorkspace),
        )?;
    }

    Ok(())
}

pub fn list<W: Write>(
    app_data: &AppData,
    printer: &mut impl Print<W>,
    print_capabilities: bool,
) -> Result<()> {
    let mut printer = printer.sub_list("Workspaces")?;
    for workspace in app_data.workspace_state.workspaces() {
        let mut printer = printer.sub_struct()?;
        printer.field("Name", &workspace.name)?;
        printer.optional("wayland id", workspace.id.as_ref())?;
        let displays =
            get_groups_for_workspace(workspace, app_data).flat_map(|wg| wg.outputs.iter());
        print_displays(app_data, &mut printer, displays)?;
        printer.field("Tiling", is_workspace_tiling(workspace))?;
        printer.field(
            "Toplevel count",
            workspace_toplevels(workspace, app_data).count(),
        )?;
        {
            use ext_workspace_handle_v1::State;
            let mut printer = printer.sub_list_with("State", ListOptions { inline: true })?;
            if workspace.state.contains(State::Active) {
                printer.item("active")?;
            }
            if workspace.state.contains(State::Hidden) {
                printer.item("hidden")?;
            }
            if workspace.state.contains(State::Urgent) {
                printer.item("urgent")?;
            }
            if workspace.state.is_empty() {
                printer.item("-")?;
            }
        }
        if print_capabilities {
            let mut printer =
                printer.sub_list_with("Capabilities", ListOptions { inline: true })?;
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Move)
            {
                printer.item("move")?;
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Pin)
            {
                printer.item("pin")?;
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Rename)
            {
                printer.item("rename")?;
            }
            if workspace
                .cosmic_capabilities
                .contains(WorkspaceCapabilities::Pin)
            {
                printer.item("set tiling")?;
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Activate)
            {
                printer.item("activate")?;
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Assign)
            {
                printer.item("assign")?;
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Deactivate)
            {
                printer.item("deactivate")?;
            }
            if workspace
                .capabilities
                .contains(ExtWorkspaceCapabilities::Remove)
            {
                printer.item("remove")?;
            }
            if workspace.capabilities.is_empty() && workspace.cosmic_capabilities.is_empty() {
                printer.item("-")?;
            }
        }
    }
    Ok(())
}

pub fn get_workspace<'a>(
    app_data: &'a AppData,
    workspace: &WorkspaceIdent,
) -> Result<(&'a WorkspaceGroup, usize, &'a Workspace)> {
    if let Some(display) = workspace.display.as_ref() {
        let Some(group) = app_data.workspace_state.workspace_groups().find(|group| {
            group
                .outputs
                .iter()
                .filter_map(|o| app_data.output_state.info(o))
                .any(|o| &output::display_name(&o) == display)
        }) else {
            bail!("Unknonw display: {}", display);
        };

        let Some((workspace_pos, workspace)) = group
            .workspaces
            .iter()
            .enumerate()
            .filter_map(|(i, handle)| {
                app_data
                    .workspace_state
                    .workspace_info(handle)
                    .map(|info| (i, info))
            })
            .find(|(_, w)| w.name == workspace.name)
        else {
            bail!(
                "Workspace {} does not exist on display {display}",
                workspace.name
            );
        };

        Ok((group, workspace_pos, workspace))
    } else {
        let mut candidate_workspaces = app_data
            .workspace_state
            .workspaces()
            .filter(|w| w.name == workspace.name);

        let Some(workspace) = candidate_workspaces.next() else {
            bail!("Workspace {} does not exist", workspace.name);
        };

        if candidate_workspaces.next().is_some() {
            bail!(
                "Found multiple workspaces with name {}. Specify display to narrow down selection",
                workspace.name
            );
        }

        let Some((workspace_pos, group)) = app_data
            .workspace_state
            .workspace_groups()
            .enumerate()
            .find(|(_, group)| group.workspaces.contains(&workspace.handle))
        else {
            bail!(
                "Found workspace {} but could not access it's group",
                workspace.name
            );
        };

        Ok((group, workspace_pos, workspace))
    }
}

pub fn pin(app_data: &AppData, workspace: WorkspaceIdent, pin: bool) -> Result<()> {
    let workspace_manager = app_data
        .workspace_state
        .workspace_manager()
        .get()
        .context("could not get acccess to workspace manager")?;

    let (_, _, workspace) = get_workspace(app_data, &workspace)?;

    let Some(cosmic_handle) = workspace.cosmic_handle.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for workspace {}",
            workspace.name
        );
    };

    if pin {
        cosmic_handle.pin();
    } else {
        cosmic_handle.unpin();
    }
    workspace_manager.commit();

    Ok(())
}

pub fn activate(app_data: &AppData, workspace: WorkspaceIdent) -> Result<()> {
    let workspace_manager = app_data
        .workspace_state
        .workspace_manager()
        .get()
        .context("could not get acccess to workspace manager")?;

    let (_, _, workspace) = get_workspace(app_data, &workspace)?;

    workspace.handle.activate();
    workspace_manager.commit();

    Ok(())
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
) -> Result<()> {
    let workspace_manager = app_data
        .workspace_state
        .workspace_manager()
        .get()
        .context("could not get acccess to workspace manager")?;

    let (orig_group, current_pos, workspace) = get_workspace(app_data, &workspace)?;

    let group = if let Some(target_display) = target_display {
        let Some(group) = app_data.workspace_state.workspace_groups().find(|group| {
            group
                .outputs
                .iter()
                .filter_map(|o| app_data.output_state.info(o))
                .any(|o| &output::display_name(&o) == target_display)
        }) else {
            bail!("Unknonw display: {}", target_display);
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
        bail!(
            "workspace {} already at position {position}",
            workspace.name
        );
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
        bail!(
            "INTERNAL: No cosmic handle for workspace {}",
            workspace.name
        );
    };

    if move_after {
        cosmic_handle.move_after(other_workspace, AXIS);
    } else {
        cosmic_handle.move_before(other_workspace, AXIS);
    }
    workspace_manager.commit();

    Ok(())
}

pub fn is_workspace_tiling(workspace: &Workspace) -> bool {
    match workspace.tiling.and_then(|s| s.into_result().ok()) {
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
