use crate::{cosmic::AppData, output};

pub fn list(app_data: &AppData, display: Option<String>) {
    let toplevels: Vec<_> = if let Some(display) = display.as_ref() {
        let Some(display) = output::find(app_data, &display) else {
            // TODO error
            return;
        };
        app_data
            .toplevel_info_state
            .toplevels()
            .filter(|t| t.output.contains(&display.0))
            .collect()
    } else {
        app_data.toplevel_info_state.toplevels().collect()
    };

    println!("Toplevels:");
    for toplevel in toplevels {
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
        println!("output count: {}", toplevel.output.len());
        println!();
    }
}
