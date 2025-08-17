use cosmic_client_toolkit::sctk::output::OutputInfo;
use wayland_client::protocol::wl_output::WlOutput;

use crate::{cosmic::AppData, print_otpion};

pub fn display_name(output: &OutputInfo) -> String {
    if let Some(name) = &output.name {
        name.clone()
    } else {
        format!("{}+{}", output.make, output.model)
    }
}

pub fn print_displays<'a, O: IntoIterator<Item = &'a WlOutput>>(app_data: &AppData, outputs: O) {
    print!("Display: ");
    let mut first = true;
    for output in outputs {
        if first {
            first = false;
        } else {
            print!(", ");
        }
        if let Some(output) = app_data.output_state.info(output) {
            print!("{}", display_name(&output));
        } else {
            print!("unknown");
        }
    }
    if first {
        print!("none");
    }
    println!();
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

pub fn list(app_data: &AppData) {
    println!("Outputs:");
    for output in app_data
        .output_state
        .outputs()
        .filter_map(|o| app_data.output_state.info(&o))
    {
        print_otpion(output.name.as_ref(), "Name");
        print_otpion(output.description.as_ref(), "Description");

        if let Some(mode) = output.modes.iter().find(|m| m.current) {
            println!("width: {}", mode.dimensions.0);
            println!("height: {}", mode.dimensions.1);
            println!("refresh: {}", mode.refresh_rate);
            println!("preferred: {}", mode.preferred);
        }

        println!("x: {}", output.location.0);
        println!("y: {}", output.location.1);
        println!("Make: {}", output.make);
        println!("Model: {}", output.model);
        println!("phys width: {}", output.physical_size.0);
        println!("phys height: {}", output.physical_size.1);

        println!();
    }
}
