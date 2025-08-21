use cosmic_client_toolkit::{
    sctk::{
        self,
        output::{OutputHandler, OutputState},
        registry::{ProvidesRegistryState, RegistryState},
    },
    toplevel_info::{ToplevelInfoHandler, ToplevelInfoState},
    toplevel_management::{ToplevelManagerHandler, ToplevelManagerState},
    workspace::{WorkspaceHandler, WorkspaceState},
};
use log::trace;
use wayland_client::QueueHandle;
use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1;

pub struct AppData {
    pub output_state: OutputState,
    pub registry_state: RegistryState,
    pub workspace_state: WorkspaceState,
    pub toplevel_info_state: ToplevelInfoState,
    pub toplevel_manager_state: ToplevelManagerState,

    pub toplevl_done: bool,
    pub workspace_done: bool,
    pub output_count: u32,
}

sctk::delegate_output!(AppData);
sctk::delegate_registry!(AppData);
cosmic_client_toolkit::delegate_workspace!(AppData);
cosmic_client_toolkit::delegate_toplevel_info!(AppData);
cosmic_client_toolkit::delegate_toplevel_manager!(AppData);

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers!(OutputState);
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        self.output_count += 1;
        trace!("new output: {output:?}");
    }

    fn update_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        trace!("update output: {output:?}");
    }

    fn output_destroyed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        self.output_count -= 1;
        trace!("destroy output: {output:?}");
    }
}

impl WorkspaceHandler for AppData {
    fn workspace_state(&mut self) -> &mut WorkspaceState {
        &mut self.workspace_state
    }

    fn done(&mut self) {
        trace!("workspace info done");
        self.workspace_done = true;
    }
}

impl ToplevelInfoHandler for AppData {
    fn toplevel_info_state(&mut self) -> &mut ToplevelInfoState {
        &mut self.toplevel_info_state
    }

    fn new_toplevel(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ExtForeignToplevelHandleV1,
    ) {
        trace!("new toplevel: {toplevel:?}");
    }

    fn update_toplevel(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ExtForeignToplevelHandleV1,
    ) {
        trace!("update toplevel: {toplevel:?}");
    }

    fn toplevel_closed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ExtForeignToplevelHandleV1,
    ) {
        trace!("closed toplevel: {toplevel:?}");
    }

    fn info_done(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>) {
        trace!("toplevel info done");
        self.toplevl_done = true;
    }
}

impl ToplevelManagerHandler for AppData {
    fn toplevel_manager_state(&mut self) -> &mut ToplevelManagerState {
        &mut self.toplevel_manager_state
    }

    fn capabilities(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
        capabilities: Vec<
            wayland_client::WEnum<cosmic_protocols::toplevel_management::v1::client::zcosmic_toplevel_manager_v1::ZcosmicToplelevelManagementCapabilitiesV1>,
        >,
    ) {
        trace!("toplevel manager cap: {capabilities:?}");
    }
}
