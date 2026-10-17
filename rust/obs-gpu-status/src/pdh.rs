//! Windows GPU utilization from the PDH `GPU Engine` counter set
//! (obs-rust/obs-studio#1022). Platform-neutral parsing and aggregation; the
//! sampler that reads the counters lives in `platform/windows.rs`.
//!
//! Each instance of `\GPU Engine(*)\Utilization Percentage` is named
//! `pid_<pid>_luid_0x<high>_0x<low>_phys_<n>_eng_<n>_engtype_<type>` and holds
//! one process's share of one engine. Task Manager's whole-adapter value is
//! the busiest engine, with each engine summed over processes.

use crate::Utilization;

/// One parsed `GPU Engine` counter instance name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdhInstance {
    pub pid: u32,
    /// The adapter LUID, `(high << 32) | low`, as DXGI reports it.
    pub luid: u64,
    pub phys: u32,
    pub eng: u32,
    pub engtype: String,
}

/// Parses an instance name; `None` when it does not have the expected shape.
pub fn parse_pdh_instance(_name: &str) -> Option<PdhInstance> {
    None
}

/// Utilization of adapter `luid` (only process `pid`'s share when `Some`)
/// from `(instance name, percent)` samples.
pub fn pdh_utilization(_samples: &[(String, f64)], _luid: u64, _pid: Option<u32>) -> Utilization {
    Utilization::Unavailable
}
