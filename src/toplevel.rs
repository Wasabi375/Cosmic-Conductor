use crate::cosmic::AppData;

pub fn list(app_data: &AppData) {
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
