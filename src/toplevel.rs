use std::io::Write;

use anyhow::{Context, Result, bail};
use cosmic_client_toolkit::toplevel_info::ToplevelInfo;
use cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1::State;
use itertools::Itertools;
use log::warn;

use crate::{
    args::WorkspaceIdent,
    cosmic::AppData,
    output,
    print::{DebugToDisplay, Print, PrintList},
    workspace::get_workspace,
};

pub fn list<W: Write>(
    app_data: &AppData,
    printer: &mut impl Print<W>,
    workspace: Option<String>,
    display: Option<String>,
    show_geometry: bool,
) -> Result<()> {
    let toplevels: Vec<_> = match (workspace, display) {
        (Some(workspace), display) => {
            let workspace_id = WorkspaceIdent {
                name: workspace,
                display,
            };
            let (_, _, workspace) = get_workspace(app_data, &workspace_id)?;
            crate::workspace::workspace_toplevels(workspace, app_data).collect()
        }
        (None, Some(display)) => {
            let Some(display) = output::find(app_data, &display) else {
                bail!("unknown display: {display}");
            };
            app_data
                .toplevel_info_state
                .toplevels()
                .filter(|t| t.output.contains(&display.0))
                .collect()
        }
        _ => app_data.toplevel_info_state.toplevels().collect(),
    };

    let mut printer = printer.sub_list("Toplevels")?;
    for toplevel in toplevels {
        let mut printer = printer.sub_struct()?;

        printer.field("Title", &toplevel.title)?;
        printer.field("AppId", &toplevel.app_id)?;
        printer.field("Unique Identifier", &toplevel.identifier)?;
        let states = toplevel.state.iter().map(DebugToDisplay);
        printer.inline_list("State", states)?;
        let workspace = toplevel
            .workspace
            .iter()
            .filter_map(|w| app_data.workspace_state.workspace_info(w))
            .map(|w| w.name.as_str())
            .exactly_one()
            .ok();
        printer.optional("workspace", workspace)?;
        let output = toplevel.output.iter().exactly_one().ok();
        let output_name = output
            .and_then(|handle| app_data.output_state.info(handle))
            .map(|o| output::display_name(&o));
        printer.optional("output", output_name)?;
        if show_geometry {
            let Some(output) = output else {
                warn!("no output found for toplevel: {}", toplevel.title);
                continue;
            };
            let Some(geometry) = toplevel.geometry.get(output) else {
                warn!("no geometry found for toplevel: {}", toplevel.title);
                continue;
            };
            let mut printer = printer.sub_struct("Geometry")?;
            printer.field("x", geometry.x)?;
            printer.field("y", geometry.y)?;
            printer.field("width", geometry.width)?;
            printer.field("height", geometry.height)?;
        }
    }

    Ok(())
}

pub fn find_from_id<'a>(app_data: &'a AppData, id: &str) -> Result<&'a ToplevelInfo> {
    let mut matches = app_data
        .toplevel_info_state
        .toplevels()
        .filter(|t| t.identifier.starts_with(id));
    match (matches.next(), matches.next()) {
        (Some(toplevel), None) => Ok(toplevel),
        (Some(_), Some(_)) => bail!("id \"{id}\" is not unique for toplevels"),
        (None, _) => bail!("Could not find toplevel with id: {id}"),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SetStateAction {
    Set,
    Unset,
    Toggle,
}

impl SetStateAction {
    pub fn from(unset: bool, toggle: bool) -> Result<Self> {
        match (unset, toggle) {
            (true, true) => bail!("Set and toggle options can't be used at the same time"),
            (true, false) => Ok(Self::Unset),
            (false, true) => Ok(Self::Toggle),
            (false, false) => Ok(Self::Set),
        }
    }
}

pub fn maximize(app_data: &AppData, id: &str, action: SetStateAction) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;
    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };
    let manager = &app_data.toplevel_manager_state.manager;

    match action {
        SetStateAction::Set => manager.set_maximized(handle),
        SetStateAction::Unset => manager.unset_maximized(handle),
        SetStateAction::Toggle => {
            if toplevel.state.contains(&State::Maximized) {
                manager.unset_maximized(handle);
            } else {
                manager.set_maximized(handle);
            }
        }
    }

    Ok(())
}

pub fn fullscreen(app_data: &AppData, id: &str, action: SetStateAction) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;
    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };
    let manager = &app_data.toplevel_manager_state.manager;

    match action {
        SetStateAction::Set => manager.set_fullscreen(handle, None),
        SetStateAction::Unset => manager.unset_fullscreen(handle),
        SetStateAction::Toggle => {
            if toplevel.state.contains(&State::Fullscreen) {
                manager.unset_fullscreen(handle);
            } else {
                manager.set_fullscreen(handle, None);
            }
        }
    }

    Ok(())
}

pub fn minimize(app_data: &AppData, id: &str, action: SetStateAction) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;
    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };
    let manager = &app_data.toplevel_manager_state.manager;

    match action {
        SetStateAction::Set => manager.set_minimized(handle),
        SetStateAction::Unset => manager.unset_minimized(handle),
        SetStateAction::Toggle => {
            if toplevel.state.contains(&State::Minimized) {
                manager.unset_minimized(handle);
            } else {
                manager.set_minimized(handle);
            }
        }
    }

    Ok(())
}

pub fn sticky(app_data: &AppData, id: &str, action: SetStateAction) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;
    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };
    let manager = &app_data.toplevel_manager_state.manager;

    match action {
        SetStateAction::Set => manager.set_sticky(handle),
        SetStateAction::Unset => manager.unset_sticky(handle),
        SetStateAction::Toggle => {
            if toplevel.state.contains(&State::Sticky) {
                manager.unset_sticky(handle);
            } else {
                manager.set_sticky(handle);
            }
        }
    }

    Ok(())
}

pub fn move_to(app_data: &AppData, id: &str, workspace: WorkspaceIdent) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;
    let (group, _, workspace) = get_workspace(app_data, &workspace)?;

    let output = group
        .outputs
        .iter()
        .exactly_one()
        .ok()
        .context("Failed to get output for workspace group")?;

    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };

    app_data
        .toplevel_manager_state
        .manager
        .move_to_ext_workspace(handle, &workspace.handle, output);

    Ok(())
}

pub fn activate(app_data: &AppData, id: &str) -> Result<()> {
    let toplevel = find_from_id(app_data, id)?;

    let seat = app_data
        .seat_state
        .seats()
        .exactly_one()
        .ok()
        .context("Could not get wayland seat")?;

    let Some(handle) = toplevel.cosmic_toplevel.as_ref() else {
        bail!(
            "INTERNAL: No cosmic handle for toplevel {}",
            toplevel.identifier
        );
    };

    app_data
        .toplevel_manager_state
        .manager
        .activate(handle, &seat);

    Ok(())
}
