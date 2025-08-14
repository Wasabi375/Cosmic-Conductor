mod args;
mod cosmic;
mod output;
mod toplevel;
mod workspace;

use args::{Arguments, Command, WorkspaceSubcommand};
use clap::Parser;
use cosmic::AppData;
use cosmic_client_toolkit::{
    sctk::{output::OutputState, registry::RegistryState},
    toplevel_info::ToplevelInfoState,
    workspace::WorkspaceState,
};
use log::{LevelFilter, debug, trace};
use simple_logger::SimpleLogger;
use wayland_client::{Connection, globals::registry_queue_init};

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
        Command::Toplevels => toplevel::list(&app_data),
        Command::Outputs => output::list(&app_data),
        Command::WorkspaceGroups => workspace::list_groups(&app_data),
        Command::Workspaces { subcommand } => match subcommand.unwrap_or_default() {
            WorkspaceSubcommand::List { capabilities } => workspace::list(&app_data, capabilities),
            WorkspaceSubcommand::MoveToPos {
                workspace,
                position,
            } => workspace::move_to(&app_data, workspace, position.into(), None),
            WorkspaceSubcommand::MoveToDisplay {
                workspace,
                target_display,
                position,
            } => workspace::move_to(
                &app_data,
                workspace,
                position.map(Into::into).unwrap_or(usize::MAX),
                Some(&target_display),
            ),
        },
    }
    event_queue.flush().unwrap();
}

pub fn print_otpion<D: std::fmt::Display>(value: Option<D>, info: &str) {
    if let Some(value) = value {
        println!("{}: {}", info, value);
    }
}
