mod args;
mod cosmic;

use args::{Arguments, Command};
use clap::Parser;
use cosmic::AppData;
use cosmic_client_toolkit::{
    sctk::{
        output::{OutputInfo, OutputState},
        registry::RegistryState,
    },
    toplevel_info::{ToplevelInfo, ToplevelInfoState},
    workspace::{Workspace, WorkspaceGroup, WorkspaceState},
};
use cosmic_protocols::workspace::v2::client::zcosmic_workspace_handle_v2::TilingState;
use log::{LevelFilter, debug, trace};
use simple_logger::SimpleLogger;
use wayland_client::{Connection, globals::registry_queue_init, protocol::wl_output::WlOutput};
use wayland_protocols::ext::workspace::v1::client::ext_workspace_group_handle_v1::GroupCapabilities;

use std::{cmp::min, thread, time::Duration};

fn main() {
    SimpleLogger::new()
        .with_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Error
        })
        .env()
        .init()
        .unwrap();

    let args = Arguments::parse();

    let connection: Connection = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();
    let qh = event_queue.handle();
    let registry_state = RegistryState::new(&globals);

    let mut app_data = AppData {
        output_state: OutputState::new(&globals, &qh),
        workspace_state: WorkspaceState::new(&registry_state, &qh),
        toplevel_info_state: ToplevelInfoState::new(&registry_state, &qh),
        registry_state,
        toplevl_done: false,
        workspace_done: false,
        output_count: 0,
    };

    let check_done = |app_data: &AppData| {
        app_data.toplevl_done && app_data.output_count > 0 && app_data.workspace_done
    };

    let mut count = 1u64;
    let mut delay = Duration::from_millis(20);
    while !check_done(&app_data) {
        if event_queue.roundtrip(&mut app_data).unwrap() == 0 {
            thread::sleep(delay);
            trace!("roundtrip sleep: {:?}", delay);
            delay = min(delay * 2, Duration::from_millis(200));
        }
        count += 1;
    }
    debug!("finished {count} wayland event roundtrips");

    match args.command {
        Command::Toplevels => toplevels(&app_data),
        Command::Outputs => outputs(&app_data),
        Command::WorkspaceGroups => workspace_groups(&app_data),
        Command::Workspaces => workspaces(&app_data),
    }
}

fn output_display_name(output: &OutputInfo) -> String {
    if let Some(name) = &output.name {
        name.clone()
    } else {
        format!("{}+{}", output.make, output.model)
    }
}

fn workspace_groups(app_data: &AppData) {
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

fn print_displays<'a, O: IntoIterator<Item = &'a WlOutput>>(app_data: &AppData, outputs: O) {
    print!("Display: ");
    let mut first = true;
    for output in outputs {
        if first {
            first = false;
        } else {
            print!(", ");
        }
        if let Some(output) = app_data.output_state.info(output) {
            print!("{}", output_display_name(&output));
        } else {
            print!("unknown");
        }
    }
    if first {
        print!("none");
    }
    println!();
}

fn workspaces(app_data: &AppData) {
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

fn is_workspace_tiling(workspace: &Workspace) -> bool {
    match workspace.tiling.map(|s| s.into_result().ok()).flatten() {
        Some(TilingState::TilingEnabled) => true,
        Some(_) | None => false,
    }
}

fn get_groups_for_workspace<'a>(
    workspace: &Workspace,
    app_data: &'a AppData,
) -> impl Iterator<Item = &'a WorkspaceGroup> {
    app_data
        .workspace_state
        .workspace_groups()
        .filter(|wg| wg.workspaces.contains(&workspace.handle))
}

fn workspace_toplevels<'a>(
    workspace: &Workspace,
    app_data: &'a AppData,
) -> impl Iterator<Item = &'a ToplevelInfo> {
    app_data
        .toplevel_info_state
        .toplevels()
        .filter(|t| t.workspace.contains(&workspace.handle))
}

fn outputs(app_data: &AppData) {
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

fn toplevels(app_data: &AppData) {
    println!("Toplevels:");
    for toplevel in app_data.toplevel_info_state.toplevels() {
        println!("Title: {}", toplevel.title);
        println!("AppId: {}", toplevel.app_id);
        println!("Unique Identifier: {}", toplevel.identifier);
        println!("State: {:?}", toplevel.state);
        let workspaces: Vec<_> = toplevel
            .workspace
            .iter()
            .filter_map(|w| app_data.workspace_state.workspace_info(w))
            .map(|w| &w.name)
            .collect();
        println!("Workspaces: {:?}", workspaces);
        println!();
    }
}

fn print_otpion<D: std::fmt::Display>(value: Option<D>, info: &str) {
    if let Some(value) = value {
        println!("{}: {}", info, value);
    }
}
