use std::io::Write;

use anyhow::Result;

use cosmic_client_toolkit::sctk::output::OutputInfo;
use wayland_client::protocol::wl_output::WlOutput;

use crate::{
    cosmic::AppData,
    print::{Print, PrintList},
};

pub fn display_name(output: &OutputInfo) -> String {
    if let Some(name) = &output.name {
        name.clone()
    } else {
        format!("{}+{}", output.make, output.model)
    }
}

pub fn print_displays<'a, O: IntoIterator<Item = &'a WlOutput>, W: Write>(
    app_data: &AppData,
    printer: &mut impl Print<W>,
    outputs: O,
) -> Result<()> {
    printer.inline_list(
        "Displays",
        outputs
            .into_iter()
            .filter_map(|handle| app_data.output_state.info(handle))
            .map(|o| display_name(&o)),
    )?;
    Ok(())
}

pub fn find(app_data: &AppData, display: &str) -> Option<(WlOutput, OutputInfo)> {
    app_data
        .output_state
        .outputs()
        .filter_map(|handle| {
            app_data
                .output_state
                .info(&handle)
                .map(|info| (handle, info))
        })
        .find(|(_, o)| &display_name(o) == display)
}

pub fn list<W: Write>(app_data: &AppData, printer: &mut impl Print<W>) -> Result<()> {
    let mut printer = printer.sub_list("Outputs")?;
    for output in app_data
        .output_state
        .outputs()
        .filter_map(|o| app_data.output_state.info(&o))
    {
        let mut printer = printer.sub_struct()?;
        printer.optional("Name", output.name.as_ref())?;
        printer.optional("Description", output.description.as_ref())?;

        if let Some(mode) = output.modes.iter().find(|m| m.current) {
            printer.field("width", mode.dimensions.0)?;
            printer.field("height", mode.dimensions.1)?;
            printer.field("refresh", mode.refresh_rate)?;
            printer.field("preferred", mode.preferred)?;
        }

        printer.field("x", output.location.0)?;
        printer.field("y", output.location.1)?;
        printer.field("Make", output.make)?;
        printer.field("Model", output.model)?;
        printer.field("phys width", output.physical_size.0)?;
        printer.field("phys height", output.physical_size.1)?;
    }

    Ok(())
}
