//! `os_get_free_disk_space` with a deadline (#1345).
//!
//! `statvfs` / `GetDiskFreeSpaceEx` on a hung network share can block for
//! minutes. obs-websocket's `GetStats` calls the export on a pool thread once
//! per request, so a client polling every second filled the pool and stopped
//! every request and event. Here each path has at most one query in flight on
//! a helper thread; a call waits for it up to the deadline, then returns the
//! path's last value (0 before the first one, which callers already read as
//! "unknown"). A responsive volume answers well inside the deadline, so its
//! result is unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How long a caller waits for a fresh answer.
pub const DEADLINE: Duration = Duration::from_millis(200);

/// The platform query: bytes free on the volume of a path.
pub type Query = Arc<dyn Fn(&[u8]) -> u64 + Send + Sync>;

pub struct BoundedFreeSpace {
    query: Query,
    deadline: Duration,
    paths: Mutex<HashMap<Vec<u8>, ()>>,
}

impl BoundedFreeSpace {
    pub fn new(query: Query, deadline: Duration) -> Self {
        Self { query, deadline, paths: Mutex::new(HashMap::new()) }
    }

    /// Bytes free on the volume of `path`, within about the deadline.
    pub fn get(&self, path: &[u8]) -> u64 {
        let _ = (&self.query, self.deadline, &self.paths, path);
        todo!("#1345")
    }
}
