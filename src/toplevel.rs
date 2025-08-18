use std::io::Write;

use anyhow::{Result, bail};
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
            crate::workspace::workspace_toplevels(&workspace, app_data).collect()
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
        let states = toplevel.state.iter().map(|s| DebugToDisplay(s));
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
            .map(|handle| app_data.output_state.info(handle))
            .flatten()
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
