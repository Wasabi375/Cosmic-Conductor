mod args;
mod cosmic;

use args::{Arguments, Command};
use clap::Parser;
use cosmic::AppData;
use log::{LevelFilter, debug};
use simple_logger::SimpleLogger;
use wayland_client::{Connection, EventQueue, Proxy, protocol::wl_display::WlDisplay};

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
    let display: WlDisplay = connection.display();
    let mut event_queue: EventQueue<AppData> = connection.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, cosmic::UserData {});

    let mut app_data = AppData::default();

    // roundtrip until we have a roundtrip with 0 events
    // I should replace this with some smart system that roundtrips until I
    // recieve the "done" events for the infos I care about
    let mut count = 0;
    while event_queue.roundtrip(&mut app_data).unwrap() != 0 {
        debug!("roundtrip done");
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

fn workspace_groups(app_data: &AppData) {
    println!("Workspace Groups:");
    for wg in &app_data.workspace_groups {
        print!("Display: ");
        let mut first = true;
        for output in &wg.outputs {
            if first {
                first = false;
            } else {
                print!(", ");
            }
            if let Some(output) = app_data.outputs.iter().find(|o| &o.handle == output) {
                print!("{}", output.display_name());
            } else {
                print!("unknown");
            }
        }
        if wg.outputs.is_empty() {
            print!("none");
        }
        println!();
        println!("workspace count: {}", wg.workspaces.len());
        println!("can create workspace: {}", wg.can_create_workspace);
    }
}

fn workspaces(app_data: &AppData) {
    let _ = app_data;
    println!("not implemented");
}

fn outputs(app_data: &AppData) {
    println!("Outputs:");
    for output in &app_data.outputs {
        print_otpion(output.name.as_ref(), "Name");
        print_otpion(output.description.as_ref(), "Description");

        if cfg!(debug_assertions) {
            println!("ObjectId: {}", output.handle.id());
        }

        if let Some(mode) = output.current_mode() {
            println!("width: {}", mode.width);
            println!("height: {}", mode.height);
            println!("refresh: {}", mode.refresh);
            println!("preferred: {}", mode.preferred);
        }

        println!("x: {}", output.x);
        println!("y: {}", output.y);
        println!("Make: {}", output.make);
        println!("Model: {}", output.model);
        println!("phys width: {}", output.phys_width);
        println!("phys height: {}", output.phys_height);

        println!();
    }
}

fn toplevels(app_data: &AppData) {
    println!("Toplevels:");
    for toplevel in &app_data.toplevels {
        print_otpion(toplevel.title.as_ref(), "Title");
        print_otpion(toplevel.app_id.as_ref(), "AppId");
        print_otpion(toplevel.ext_id.as_ref(), "Unique Identifier");
        if cfg!(debug_assertions) {
            println!("ObjectId: {}", toplevel.handle.id());
        }
        println!("State: {:?}", toplevel.state);
        println!();
    }
}

fn print_otpion<D: std::fmt::Display>(value: Option<D>, info: &str) {
    if let Some(value) = value {
        println!("{}: {}", info, value);
    }
}
