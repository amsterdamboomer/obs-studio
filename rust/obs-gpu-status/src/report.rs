//! What the status light and its dialog show, and what obs-websocket's
//! `GetStats` returns as `gpuAcceleration` (obs-rust/obs-studio#846).

use crate::{FrameStats, GpuDevice, State, Vendor, classify};

/// The classified state plus the plain-words text the dialog shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub state: State,
    /// The active adapter, `None` when graphics initialization failed.
    pub adapter: Option<String>,
    pub devices: Vec<GpuDevice>,
    /// One or two sentences: what is happening and what it means.
    pub summary: String,
    /// Likely causes to check, empty when there is nothing to fix.
    pub causes: Vec<String>,
}

impl Report {
    pub fn new(adapter: Option<&str>, devices: Vec<GpuDevice>, frames: Option<FrameStats>) -> Self {
        todo!("{adapter:?} {devices:?} {frames:?}")
    }

    /// The `gpuAcceleration` JSON object:
    /// `{"state","adapter","detectedGpus":[{"vendor","deviceId","name","renderNode"}],"summary","causes"}`.
    pub fn to_json(&self) -> String {
        todo!("{self:?}")
    }
}

impl State {
    /// `"green"`, `"yellow"` or `"red"`.
    pub fn as_str(self) -> &'static str {
        todo!("{self:?}")
    }
}

impl Vendor {
    /// `"nvidia"`, `"amd"` or `"intel"`.
    pub fn as_str(self) -> &'static str {
        todo!("{self:?}")
    }
}
