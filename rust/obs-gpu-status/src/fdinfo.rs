//! Per-process GPU utilization from DRM fdinfo (obs-rust/obs-studio#1023):
//! the `drm-*` keys the kernel adds to `/proc/<pid>/fdinfo/<fd>` of a DRM
//! file (docs.kernel.org/gpu/drm-usage-stats.html). Covers i915, xe and
//! amdgpu where the kernel exposes engine times.

use std::collections::BTreeMap;
use std::path::Path;

use crate::Utilization;

/// One engine's counters in one fdinfo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Engine {
    /// `drm-engine-<e>` in ns, or xe's `drm-cycles-<e>`.
    pub busy: u64,
    /// xe's `drm-total-cycles-<e>`: the clock `busy` counts against. `None`
    /// for ns engines, which count against wall time.
    pub total: Option<u64>,
    /// `drm-engine-capacity-<e>`: how many engines of the class; 1 if absent.
    pub capacity: u64,
}

/// The DRM keys of one fdinfo file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdinfoClient {
    pub driver: String,
    /// `drm-pdev`: the PCI address of the device, if the driver gives one.
    pub pdev: Option<String>,
    /// `drm-client-id`: fds that share one (dup'ed or passed) are one client.
    pub client_id: u64,
    pub engines: BTreeMap<String, Engine>,
}

/// Parse one fdinfo text; `None` if it is not a DRM client with an id.
pub fn parse_fdinfo(_text: &str) -> Option<FdinfoClient> {
    None
}

/// Every DRM client among the fdinfo files in `dir` (`/proc/self/fdinfo`).
pub fn read_fdinfo(_dir: &Path) -> Vec<FdinfoClient> {
    Vec::new()
}

/// The busiest engine's utilization on `pdev` between two samples taken
/// `elapsed_ns` apart, each client counted once.
pub fn fdinfo_utilization(
    _before: &[FdinfoClient],
    _after: &[FdinfoClient],
    _pdev: &str,
    _elapsed_ns: u64,
) -> Utilization {
    Utilization::Unavailable
}
