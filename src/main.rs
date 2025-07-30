mod cosmic;

use cosmic::AppData;
use log::{LevelFilter, debug};
use simple_logger::SimpleLogger;
use wayland_client::{Connection, EventQueue, protocol::wl_display::WlDisplay};

fn main() {
    SimpleLogger::new()
        .with_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
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

    let mut count = 0;
    while event_queue.roundtrip(&mut app_data).unwrap() != 0 {
        debug!("roundtrip done");
        count += 1;
    }

    assert_eq!(count, 3);
}
