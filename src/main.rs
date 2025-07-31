mod cosmic;

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

    assert_eq!(count, 2);

    println!("Toplevels:");
    for toplevel in app_data.toplevels {
        print_otpion(toplevel.title, "Title");
        print_otpion(toplevel.app_id, "AppId");
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
