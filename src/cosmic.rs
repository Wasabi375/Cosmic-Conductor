#![allow(dead_code)]
#![warn(unused_imports)]

use cosmic_protocols::{
    toplevel_info::v1::client::{
        zcosmic_toplevel_handle_v1::{self, ZcosmicToplevelHandleV1},
        zcosmic_toplevel_info_v1::{self, ZcosmicToplevelInfoV1},
    },
    workspace::v2::client::{
        zcosmic_workspace_handle_v2, zcosmic_workspace_manager_v2::ZcosmicWorkspaceManagerV2,
    },
};
use log::{debug, info, warn};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, event_created_child,
    protocol::{
        wl_output,
        wl_registry::{self, WlRegistry},
    },
};

#[derive(Debug, Clone, Default)]
pub struct UserData {}

pub struct CosmicTopLevelInfo {
    info: ZcosmicToplevelInfoV1,
    name: u32,
}

pub const TOPLEVEL_HANDLE_DISPLAY_NAME: &str = "COSMIC toplevel handle";

impl CosmicTopLevelInfo {
    pub const INTERFACE: &str = "zcosmic_toplevel_info_v1";
    pub const DISPLAY_NAME: &str = "COSMIC toplevel info";

    pub fn bind(
        registry: &wl_registry::WlRegistry,
        name: u32,
        qh: &QueueHandle<AppData>,
        udata: UserData,
    ) -> Self {
        let info = registry.bind::<ZcosmicToplevelInfoV1, _, _>(name, 1, qh, udata);

        debug!("bind: {info:?}");

        Self { info, name }
    }

    pub fn name(&self) -> u32 {
        self.name
    }

    pub fn protocol(&self) -> &ZcosmicToplevelInfoV1 {
        &self.info
    }
}

pub struct CosmicWorkspaceManager {
    manager: ZcosmicWorkspaceManagerV2,
    name: u32,
}
impl CosmicWorkspaceManager {
    pub const INTERFACE: &str = "zcosmic_workspace_manager_v2";
    pub const DISPLAY_NAME: &str = "COSMIC workspace manager";

    pub fn bind(
        registry: &wl_registry::WlRegistry,
        name: u32,
        qh: &QueueHandle<AppData>,
        udata: UserData,
    ) -> Self {
        let manager = registry.bind::<ZcosmicWorkspaceManagerV2, _, _>(name, 1, qh, udata);

        debug!("bind: {manager:?}");

        Self { manager, name }
    }

    pub fn name(&self) -> u32 {
        self.name
    }

    pub fn protocol(&self) -> &ZcosmicWorkspaceManagerV2 {
        &self.manager
    }
}

#[derive(Default)]
pub struct AppData {
    pub toplevel_info: Option<CosmicTopLevelInfo>,
    pub workspace_manager: Option<CosmicWorkspaceManager>,
    pub outputs: Vec<(wl_output::WlOutput, String)>,
    pub toplevels: Vec<Toplevel>,
    pub workspaces: Vec<(
        Vec<(
            zcosmic_workspace_handle_v2::ZcosmicWorkspaceHandleV2,
            Option<String>,
        )>,
    )>,
}

impl Dispatch<WlRegistry, UserData> for AppData {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _udata: &UserData,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wl_registry::Event;
        info!(target: "WlRegistry", "event: {event:?}");
        match event {
            Event::Global {
                name,
                interface,
                version: _,
            } => match &*interface {
                CosmicTopLevelInfo::INTERFACE => {
                    state.toplevel_info = Some(CosmicTopLevelInfo::bind(
                        registry,
                        name,
                        qh,
                        Default::default(),
                    ));
                }
                CosmicWorkspaceManager::INTERFACE => {
                    state.workspace_manager = Some(CosmicWorkspaceManager::bind(
                        registry,
                        name,
                        qh,
                        Default::default(),
                    ));
                }
                _ => {
                    // we don't care about this interface
                }
            },
            Event::GlobalRemove { name } => {
                if let Some(proxy) = &state.toplevel_info
                    && proxy.name() == name
                {
                    info!("{} removed", CosmicTopLevelInfo::DISPLAY_NAME);
                    state.toplevel_info = None;
                }
                if let Some(proxy) = &state.workspace_manager
                    && proxy.name() == name
                {
                    info!("{} removed", CosmicWorkspaceManager::DISPLAY_NAME);
                    state.workspace_manager = None;
                }
            }
            _ => {
                warn!("unknown event: {event:?}");
                return;
            }
        }
    }
}

impl Dispatch<ZcosmicToplevelInfoV1, UserData> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZcosmicToplevelInfoV1,
        event: <ZcosmicToplevelInfoV1 as wayland_client::Proxy>::Event,
        data: &UserData,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        warn!(target: CosmicTopLevelInfo::DISPLAY_NAME, "not implemented: handle event: {event:?}")
    }

    event_created_child!(
        AppData,
        ZcosmicToplevelInfoV1,
        [
            zcosmic_toplevel_info_v1::EVT_TOPLEVEL_OPCODE => (ZcosmicToplevelHandleV1, UserData::default()),
        ]
    );
}

impl Dispatch<ZcosmicToplevelHandleV1, UserData> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZcosmicToplevelHandleV1,
        event: <ZcosmicToplevelHandleV1 as Proxy>::Event,
        data: &UserData,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        warn!(target: TOPLEVEL_HANDLE_DISPLAY_NAME, "not implemented: handle event: {event:?}")
    }
}

impl Dispatch<ZcosmicWorkspaceManagerV2, UserData> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZcosmicWorkspaceManagerV2,
        event: <ZcosmicWorkspaceManagerV2 as Proxy>::Event,
        data: &UserData,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        warn!(target: CosmicWorkspaceManager::DISPLAY_NAME, "not implemented: handle event: {event:?}")
    }
}

pub struct Toplevel {
    pub handle: zcosmic_toplevel_handle_v1::ZcosmicToplevelHandleV1,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub outputs: Vec<wl_output::WlOutput>,
    pub workspaces: Vec<zcosmic_workspace_handle_v2::ZcosmicWorkspaceHandleV2>,
    pub state: Vec<State>,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    Maximized = 0,
    Minimized = 1,
    Activated = 2,
    Fullscreen = 3,
}

impl TryFrom<u32> for State {
    type Error = ();
    fn try_from(val: u32) -> Result<State, ()> {
        match val {
            0 => Ok(State::Maximized),
            1 => Ok(State::Minimized),
            2 => Ok(State::Activated),
            3 => Ok(State::Fullscreen),
            _ => Err(()),
        }
    }
}
