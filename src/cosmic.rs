#![allow(dead_code)]
#![warn(unused_imports)]

use cosmic_protocols::{
    toplevel_info::v1::client::{
        zcosmic_toplevel_handle_v1::{self, ZcosmicToplevelHandleV1},
        zcosmic_toplevel_info_v1::{self, ZcosmicToplevelInfoV1},
    },
    workspace::v2::client::{
        zcosmic_workspace_handle_v2::ZcosmicWorkspaceHandleV2,
        zcosmic_workspace_manager_v2::ZcosmicWorkspaceManagerV2,
    },
};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, event_created_child,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
};

#[allow(unused)]
use log::{debug, error, info, warn};

#[derive(Debug, Clone, Default)]
pub struct UserData {}

pub struct CosmicTopLevelInfo {
    info: ZcosmicToplevelInfoV1,
    name: u32,
}

pub const TOPLEVEL_HANDLE_DISPLAY_NAME: &str = "COSMIC toplevel handle";

pub const WL_OUTPUT_INTERFACE: &str = "wl_output";
pub const WL_OUTPUT_DISPLAY_NAME: &str = "Wl Output";

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
    pub outputs: Vec<Output>,
    pub toplevels: Vec<Toplevel>,
    pub workspaces: Vec<(Vec<(ZcosmicWorkspaceHandleV2, Option<String>)>,)>,
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
        debug!(target: "WlRegistry", "event: {event:?}");
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
                WL_OUTPUT_INTERFACE => {
                    registry.bind::<WlOutput, UserData, _>(name, 4, qh, UserData::default());
                }
                _ => {
                    //                     if interface.contains("wl")
                    //                         || interface.contains("cosmic")
                    //                         || interface.contains("workspace")
                    //                     {
                    //                         warn!("unused wl/cosmic interface: {interface}");
                    //                     }
                    // we don't care about this interface
                }
            },
            Event::GlobalRemove { name } => {
                if let Some(proxy) = &state.toplevel_info
                    && proxy.name() == name
                {
                    debug!("{} removed", CosmicTopLevelInfo::DISPLAY_NAME);
                    state.toplevel_info = None;
                }
                if let Some(proxy) = &state.workspace_manager
                    && proxy.name() == name
                {
                    debug!("{} removed", CosmicWorkspaceManager::DISPLAY_NAME);
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
        app_data: &mut Self,
        _proxy: &ZcosmicToplevelInfoV1,
        event: <ZcosmicToplevelInfoV1 as wayland_client::Proxy>::Event,
        _udata: &UserData,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        use zcosmic_toplevel_info_v1::Event;
        debug!(target: CosmicTopLevelInfo::DISPLAY_NAME, "event: {event:?}");

        match event {
            Event::Toplevel { toplevel: handle } => {
                app_data.toplevels.push(Toplevel::new(handle));
            }
            Event::Finished => {
                app_data.toplevel_info = None;
            }
            Event::Done => {
                // all info about active toplevels recieved. We don't need to do anything
            }
            _ => warn!("unknown event: {event:?}"),
        }
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
        app_data: &mut Self,
        toplevel: &ZcosmicToplevelHandleV1,
        event: <ZcosmicToplevelHandleV1 as Proxy>::Event,
        _udata: &UserData,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        use zcosmic_toplevel_handle_v1::Event;
        debug!(target: TOPLEVEL_HANDLE_DISPLAY_NAME, "handle event: {event:?}");

        let Some(toplevel) = app_data
            .toplevels
            .iter_mut()
            .find(|t| &t.handle == toplevel)
        else {
            error!("Got unknown toplevel handle: {}", toplevel.id());
            return;
        };

        match event {
            Event::Closed => {}
            Event::Done => {}
            Event::Title { title } => toplevel.title = Some(title),
            Event::AppId { app_id } => toplevel.app_id = Some(app_id),
            Event::OutputEnter { output } => {
                debug!(
                    "{:?}({}) enter output {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    output.id()
                );
                // TODO
            }
            Event::OutputLeave { output } => {
                debug!(
                    "{:?}({}) leave output {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    output.id()
                ); // TODO
            }
            Event::WorkspaceEnter { workspace } => {
                debug!(
                    "{:?}({}) enter workspace {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    workspace.id()
                );
                // TODO
            }
            Event::WorkspaceLeave { workspace } => {
                debug!(
                    "{:?}({}) leave workspace {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    workspace.id()
                );
                // TODO
            }
            Event::State { state } => {
                toplevel.state = state
                    .chunks_exact(4)
                    .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
                    .flat_map(|val| ToplevelState::try_from(val).ok())
                    .collect::<Vec<_>>()
            }
            Event::Geometry {
                output,
                x,
                y,
                width,
                height,
            } => {
                debug!(
                    "{:?}({}) geo: x: {}, y: {}, w: {}, h: {}, out: {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    x,
                    y,
                    width,
                    height,
                    output.id()
                )
                // TODO
            }
            Event::ExtWorkspaceEnter { workspace } => {
                debug!(
                    "{:?}({}) enter workspace ext {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    workspace.id()
                );
                // TODO
            }
            Event::ExtWorkspaceLeave { workspace } => {
                debug!(
                    "{:?}({}) leave workspace ext {}",
                    toplevel.title,
                    toplevel.handle.id(),
                    workspace.id()
                );
                // TODO
            }
            _ => warn!("unknown event: {event:?}"),
        }
    }
}

impl Dispatch<ZcosmicWorkspaceManagerV2, UserData> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZcosmicWorkspaceManagerV2,
        event: <ZcosmicWorkspaceManagerV2 as Proxy>::Event,
        _data: &UserData,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        warn!(target: CosmicWorkspaceManager::DISPLAY_NAME, "not implemented: handle event: {event:?}");
    }
}

impl Dispatch<WlOutput, UserData> for AppData {
    fn event(
        app_data: &mut Self,
        handle: &WlOutput,
        event: <WlOutput as Proxy>::Event,
        _udata: &UserData,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        use wl_output::Event;

        let output = app_data.outputs.iter_mut().find(|o| &o.handle == handle);

        match event {
            Event::Geometry {
                x,
                y,
                physical_width,
                physical_height,
                subpixel: _,
                make,
                model,
                transform: _,
            } => {
                if let Some(output) = output {
                    output.x = x;
                    output.y = y;
                    output.phys_width = physical_width;
                    output.phys_height = physical_height;
                    output.make = make;
                    output.model = model;
                } else {
                    app_data.outputs.push(Output::new(
                        handle.clone(),
                        x,
                        y,
                        physical_width,
                        physical_height,
                        make,
                        model,
                    ));
                }
            }
            Event::Mode {
                flags,
                width,
                height,
                refresh,
            } => {
                let Some(output) = output else {
                    error!("Got unknown output handle: {}", handle.id());
                    return;
                };
                output
                    .modes
                    .push(OutputMode::new(width, height, refresh, flags));
            }
            Event::Done => {
                // all info about output recieved, we can just ignore this
            }
            Event::Scale { factor: _ } => {
                // dont care
            }
            Event::Name { name } => {
                let Some(output) = output else {
                    error!("Got unknown output handle: {}", handle.id());
                    return;
                };
                output.name = Some(name);
            }
            Event::Description { description } => {
                let Some(output) = output else {
                    error!("Got unknown output handle: {}", handle.id());
                    return;
                };
                output.description = Some(description);
            }
            _ => todo!(),
        }
    }
}

pub struct Toplevel {
    pub handle: ZcosmicToplevelHandleV1,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub outputs: Vec<wl_output::WlOutput>,
    pub workspaces: Vec<ZcosmicWorkspaceHandleV2>,
    pub state: Vec<ToplevelState>,
}

impl Toplevel {
    pub fn new(handle: ZcosmicToplevelHandleV1) -> Self {
        Self {
            handle,
            title: Default::default(),
            app_id: Default::default(),
            outputs: Default::default(),
            workspaces: Default::default(),
            state: Default::default(),
        }
    }
}

pub struct Output {
    pub handle: WlOutput,
    pub name: Option<String>,
    pub description: Option<String>,
    /// x position of output in compositor space
    pub x: i32,
    /// y position of output in compositor space
    pub y: i32,
    /// size of output in millimeters, can be 0
    pub phys_width: i32,
    /// size of output in millimeters, can be 0
    pub phys_height: i32,
    pub make: String,
    pub model: String,

    pub modes: Vec<OutputMode>,
}

impl Output {
    pub fn new(
        handle: WlOutput,
        x: i32,
        y: i32,
        phys_width: i32,
        phys_height: i32,
        make: String,
        model: String,
    ) -> Self {
        Self {
            handle,
            name: None,
            description: None,
            x,
            y,
            phys_width,
            phys_height,
            make,
            model,
            modes: Vec::new(),
        }
    }

    pub fn current_mode(&self) -> Option<&OutputMode> {
        self.modes.iter().find(|m| m.current)
    }
}

#[derive(Debug, Clone)]
pub struct OutputMode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
    pub current: bool,
    pub preferred: bool,
}

impl OutputMode {
    pub fn new(width: i32, height: i32, refresh: i32, flags: WEnum<wl_output::Mode>) -> Self {
        use wl_output::Mode;

        let (current, preferred) = if let Ok(flags) = flags.into_result() {
            (
                flags.contains(Mode::Current),
                flags.contains(Mode::Preferred),
            )
        } else {
            (false, false)
        };

        Self {
            width,
            height,
            refresh,
            preferred,
            current,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ToplevelState {
    Maximized = 0,
    Minimized = 1,
    Activated = 2,
    Fullscreen = 3,
}

impl TryFrom<u32> for ToplevelState {
    type Error = ();
    fn try_from(val: u32) -> Result<ToplevelState, ()> {
        match val {
            0 => Ok(ToplevelState::Maximized),
            1 => Ok(ToplevelState::Minimized),
            2 => Ok(ToplevelState::Activated),
            3 => Ok(ToplevelState::Fullscreen),
            _ => Err(()),
        }
    }
}
