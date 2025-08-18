use std::io::Write;

use anyhow::{Result, bail};
use itertools::Itertools;

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
        let output_name = toplevel
            .output
            .iter()
            .filter_map(|handle| app_data.output_state.info(handle))
            .map(|o| output::display_name(&o))
            .exactly_one()
            .ok();
        printer.optional("output", output_name)?;
    }

    Ok(())
}
