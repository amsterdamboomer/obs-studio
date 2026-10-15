//! GPU acceleration status for the status-bar light (obs-rust/obs-studio#846).
//!
//! [`classify`] is pure: it takes the active adapter name (as libobs logs it
//! in `Loading up OpenGL on adapter ...`), the hardware GPUs detected on the
//! system and optional frame timings, and returns green, yellow or red.
//! Detection reads a sysfs/`/dev/dri` tree under a caller-supplied root, so
//! tests use a fixture tree; [`platform::system_devices`] probes the host.

use std::path::Path;
use std::process::Command;

pub mod platform;

/// The light's color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// The active adapter is a hardware GPU.
    Green,
    /// A hardware GPU is present, but rendering runs on a software renderer.
    Yellow,
    /// Software rendering with no usable GPU, graphics init failed, or
    /// frames are behind budget.
    Red,
}

/// PCI vendor of a detected GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Nvidia,
    Amd,
    Intel,
}

impl Vendor {
    /// Map a PCI vendor ID (`0x10de`, `10de`, ...) to a known GPU vendor.
    pub fn from_pci_id(id: &str) -> Option<Self> {
        todo!("{id}")
    }
}

/// A hardware GPU found on the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDevice {
    pub vendor: Vendor,
    /// PCI device ID, lowercase hex without `0x` (e.g. `2504`), if known.
    pub device_id: Option<String>,
    /// Marketing name, if known (from `nvidia-smi`).
    pub name: Option<String>,
    /// Whether a `/dev/dri/renderD*` node belongs to this device.
    pub render_node: bool,
}

/// Graphics-thread timings over a window, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameStats {
    /// Median `obs_graphics_thread` time over the window.
    pub median_frame_ns: u64,
    /// The frame interval (budget), e.g. 33_333_333 at 30 fps.
    pub interval_ns: u64,
}

/// Whether `adapter` names a software renderer (llvmpipe, softpipe,
/// Microsoft Basic Render Driver, SwiftShader).
pub fn is_software_renderer(adapter: &str) -> bool {
    todo!("{adapter}")
}

/// Classify the GPU acceleration state.
///
/// `adapter` is `None` when graphics initialization failed.
pub fn classify(adapter: Option<&str>, devices: &[GpuDevice], frames: Option<FrameStats>) -> State {
    todo!("{adapter:?} {devices:?} {frames:?}")
}

/// Detect hardware GPUs from `<root>/sys/class/drm/card*/device/vendor`.
/// A device has a render node when a `<root>/sys/class/drm/renderD*` entry
/// has the same vendor and device ID and `<root>/dev/dri/renderD*` exists.
pub fn detect_devices(root: &Path) -> Vec<GpuDevice> {
    todo!("{}", root.display())
}

/// Parse `nvidia-smi --query-gpu=name,pci.device_id --format=csv,noheader`.
///
/// Each line is `<name>, 0x<device><vendor>` (e.g. `NVIDIA GeForce RTX 3060,
/// 0x250410DE`). Lines that do not parse are skipped.
pub fn parse_nvidia_smi(output: &str) -> Vec<GpuDevice> {
    todo!("{output}")
}

/// Run `nvidia-smi`; an absent tool or a failing driver yields no devices.
pub fn query_nvidia_smi() -> Vec<GpuDevice> {
    match Command::new("nvidia-smi")
        .args(["--query-gpu=name,pci.device_id", "--format=csv,noheader"])
        .output()
    {
        Ok(out) if out.status.success() => parse_nvidia_smi(&String::from_utf8_lossy(&out.stdout)),
        _ => Vec::new(),
    }
}

/// Add `extra` devices not already in `devices` (same vendor and device ID);
/// a match fills in the missing name instead.
pub fn merge_devices(devices: &mut Vec<GpuDevice>, extra: Vec<GpuDevice>) {
    todo!("{devices:?} {extra:?}")
}
