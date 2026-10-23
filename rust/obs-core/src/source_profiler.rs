//! RED: stubs; the port follows.
//!
//! Safe core of `libobs/util/source-profiler.c.h`: per-source tick and
//! render timing, and the fps of async sources.
//!
//! The graphics thread drives it once per frame, as `obs-video.c` does:
//! [`SourceProfiler::frame_begin`], the tick and render hooks of every
//! source, [`SourceProfiler::frame_collect`]. A frame's samples are kept in
//! a [`FRAME_BUFFER_SIZE`] ring per source and collected one frame later,
//! which gives the GPU timers time to finish. Collection pushes into the
//! source's [`Entry`]: fixed-size rings ([`CircleBuf`]) of the last five
//! seconds of frames. [`SourceProfiler::fill_result`] turns an entry into a
//! [`ProfilerResult`] for any thread.
//!
//! Everything libobs supplies (the clock, GPU timers, the graphics context,
//! a source's flags and last rendered async timestamp, the log line and
//! the profiler scope of a collection) goes through [`Host`]. Sources and
//! GPU timers are opaque [`Handle`]s; 0 is NULL.
//!
//! Kept as the C has it, characterized and not endorsed:
//!
//! - a source that is not ticked in a frame keeps its last tick in the
//!   sample ring, and collection pushes that stale tick again;
//! - GPU timers left over after GPU timing was disabled are destroyed
//!   without the graphics context (the device ignores that);
//! - sums and averages wrap like the C's `uint64_t`;
//! - `gpu_enable(true)` while the profiler is not enabled is dropped.
//!
//! Intentional differences (also in rust/README.md and #1886):
//!
//! - a [`CircleBuf`] of capacity 0 drops pushes, and a GPU frequency of 0
//!   converts to 0 ns; the C divides by zero in both;
//! - removing a source takes the entry table's write lock; the C mutates it
//!   under the read lock;
//! - the graphics context is never waited for while the sample table is
//!   locked: timers of destroyed samples are destroyed after the lock is
//!   released, in one enter/leave instead of one per sample.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use obs_util::util_uint64::mul_div64;

/// Frames a sample waits before it is collected: `NUM_TEXTURES` in
/// `obs-internal.h`, the depth of the render buffer.
pub const FRAME_BUFFER_SIZE: usize = 2;

/// Sources are rendered more than once per frame at times; room for two
/// passes is kept per frame sample.
const RENDER_TIMES_RESERVATION: usize = 2;

/// An opaque libobs pointer (a source, a `gs_timer_t`, a
/// `gs_timer_range_t`) as an address. 0 is NULL.
pub type Handle = usize;

/// What `gs_timer_range_get_data` reports. The C starts from `false` and 0
/// and keeps them when the device does not write them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RangeData {
    pub disjoint: bool,
    pub frequency: u64,
}

/// What the profiler needs from libobs.
pub trait Host {
    /// `os_gettime_ns`.
    fn now_ns(&self) -> u64;
    /// `gs_enter_context(obs->video.graphics)`.
    fn enter_graphics(&self);
    /// `gs_leave_context`.
    fn leave_graphics(&self);
    /// `gs_timer_create`; 0 for NULL.
    fn timer_create(&self) -> Handle;
    fn timer_begin(&self, timer: Handle);
    fn timer_end(&self, timer: Handle);
    /// `gs_timer_get_data`: the ticks, or `None` when it returns false.
    fn timer_get_data(&self, timer: Handle) -> Option<u64>;
    fn timer_destroy(&self, timer: Handle);
    /// `gs_timer_range_create`; 0 for NULL.
    fn range_create(&self) -> Handle;
    fn range_begin(&self, range: Handle);
    fn range_end(&self, range: Handle);
    fn range_get_data(&self, range: Handle) -> RangeData;
    fn range_destroy(&self, range: Handle);
    /// The source's output flags include `OBS_SOURCE_ASYNC_VIDEO`.
    fn is_async_video(&self, source: Handle) -> bool;
    /// `obs_source_get_last_async_ts`.
    fn last_async_ts(&self, source: Handle) -> u64;
    /// The "GPU Timers were disjoint, discarding samples." warning.
    fn warn_disjoint(&self);
    /// `profile_start` / `profile_end` of `source_profiler_frame_collect`.
    fn collect_scope_begin(&self);
    fn collect_scope_end(&self);
}

/// `struct ucirclebuf`: the most recent `capacity` values; older ones are
/// overwritten. [`CircleBuf::values`] is the storage order, as the C walks
/// `array[0..num]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CircleBuf {
    idx: usize,
    capacity: usize,
    values: Vec<u64>,
}

impl CircleBuf {
    /// `ucirclebuf_init`.
    pub fn new(capacity: usize) -> Self {
        todo!("source-profiler port")
    }

    /// `ucirclebuf_push`. Capacity 0 drops the value (the C divides by
    /// zero).
    pub fn push(&mut self, value: u64) {
        todo!("source-profiler port")
    }

    /// `array[0..num]`.
    pub fn values(&self) -> &[u64] {
        todo!("source-profiler port")
    }

    /// The next write position before wrapping (`idx`).
    pub fn idx(&self) -> usize {
        todo!("source-profiler port")
    }

    pub fn capacity(&self) -> usize {
        todo!("source-profiler port")
    }

    /// `num`.
    pub fn len(&self) -> usize {
        todo!("source-profiler port")
    }

    pub fn is_empty(&self) -> bool {
        todo!("source-profiler port")
    }
}

/// `profiler_result_t`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ProfilerResult {
    pub tick_avg: u64,
    pub tick_max: u64,
    pub render_avg: u64,
    pub render_max: u64,
    pub render_gpu_avg: u64,
    pub render_gpu_max: u64,
    pub render_sum: u64,
    pub render_gpu_sum: u64,
    pub async_input: f64,
    pub async_rendered: f64,
    pub async_input_best: u64,
    pub async_input_worst: u64,
    pub async_rendered_best: u64,
    pub async_rendered_worst: u64,
}

/// Average and maximum of `values` (`calculate_tick`): `None` for no
/// values. The sum wraps like the C's.
pub fn average_max(values: &[u64]) -> Option<(u64, u64)> {
    todo!("source-profiler port")
}

/// `calculate_tick`: writes `tick_avg` and `tick_max`.
pub fn calculate_tick(ticks: &[u64], result: &mut ProfilerResult) {
    todo!("source-profiler port")
}

/// The average, maximum and average sum of a first-pass ring and its
/// sum-of-passes ring, walked to the length of the first (the C indexes
/// the sum ring by the first ring's `num`; both are pushed together).
fn render_stats(first: &[u64], sums: &[u64]) -> Option<(u64, u64, u64)> {
    todo!("source-profiler port")
}

/// `calculate_render`: the CPU fields, and the GPU fields when `gpu` is
/// given (GPU timing enabled).
pub fn calculate_render(
    cpu: &[u64],
    cpu_sum: &[u64],
    gpu: Option<(&[u64], &[u64])>,
    result: &mut ProfilerResult,
) {
    todo!("source-profiler port")
}

/// What `calculate_fps` reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FpsStats {
    pub fps: f64,
    pub best: u64,
    pub worst: u64,
}

/// `calculate_fps` over timestamps in storage order: each timestamp's
/// delta to the one stored before it (the first to the last), skipping
/// zero or non-increasing pairs and stopping at the first zero timestamp.
/// `None` (the C writes nothing) when no delta counted or they sum to 0.
pub fn calculate_fps(frames: &[u64]) -> Option<FpsStats> {
    todo!("source-profiler port")
}

/// `struct frame_sample`: one frame of one source.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrameSample {
    /// Tick time; 0 is "no data yet".
    pub tick: u64,
    /// CPU time of each render pass.
    pub render_cpu: Vec<u64>,
    /// GPU timer of each render pass.
    pub render_timers: Vec<Handle>,
}

impl FrameSample {
    fn new() -> Self {
        todo!("source-profiler port")
    }
}

/// `struct source_samples`: a source's last [`FRAME_BUFFER_SIZE`] frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSamples {
    /// The frame being written.
    pub frame_idx: usize,
    pub frames: [FrameSample; FRAME_BUFFER_SIZE],
}

impl SourceSamples {
    fn new() -> Self {
        todo!("source-profiler port")
    }

    fn current(&mut self) -> Option<&mut FrameSample> {
        todo!("source-profiler port")
    }

    /// Every GPU timer still held, which destroying the samples destroys.
    fn take_timers(&mut self, out: &mut Vec<Handle>) {
        todo!("source-profiler port")
    }
}

/// `struct profiler_entry`: a source's collected frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Tick times.
    pub tick: CircleBuf,
    /// Time of the first render pass of a frame.
    pub render_cpu: CircleBuf,
    pub render_gpu: CircleBuf,
    /// Sum of all render passes of a frame.
    pub render_cpu_sum: CircleBuf,
    pub render_gpu_sum: CircleBuf,
    /// Timestamps of async frame submissions.
    pub async_frame_ts: CircleBuf,
    /// Timestamps of the async frames rendered, one per frame.
    pub async_rendered_ts: CircleBuf,
}

impl Entry {
    fn new(capacity: usize) -> Self {
        todo!("source-profiler port")
    }
}

/// What [`SourceProfiler::fill_result`] found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    /// The profiler is not enabled: false, the result is left alone.
    Disabled,
    /// No entry for the source: false, the result is zeroed.
    Missing,
    /// True, with the result.
    Found(ProfilerResult),
}

/// The graphics thread's state: `hm_samples` and the GPU timer ranges.
#[derive(Debug)]
struct Graphics {
    samples: BTreeMap<Handle, SourceSamples>,
    timer_idx: usize,
    timer_ranges: [Handle; FRAME_BUFFER_SIZE],
}

/// The profiler (the C file's statics).
#[derive(Debug)]
pub struct SourceProfiler {
    enabled: AtomicBool,
    gpu_enabled: AtomicBool,
    enable_next: AtomicBool,
    gpu_enable_next: AtomicBool,
    samples_per_entry: AtomicU64,
    graphics: Mutex<Graphics>,
    entries: RwLock<BTreeMap<Handle, Entry>>,
}

impl Default for SourceProfiler {
    fn default() -> Self {
        todo!("source-profiler port")
    }
}

impl SourceProfiler {
    pub const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            gpu_enabled: AtomicBool::new(false),
            enable_next: AtomicBool::new(false),
            gpu_enable_next: AtomicBool::new(false),
            samples_per_entry: AtomicU64::new(0),
            graphics: Mutex::new(Graphics {
                samples: BTreeMap::new(),
                timer_idx: 0,
                timer_ranges: [0; FRAME_BUFFER_SIZE],
            }),
            entries: RwLock::new(BTreeMap::new()),
        }
    }

    fn graphics(&self) -> MutexGuard<'_, Graphics> {
        todo!("source-profiler port")
    }

    fn entries_read(&self) -> RwLockReadGuard<'_, BTreeMap<Handle, Entry>> {
        todo!("source-profiler port")
    }

    fn entries_write(&self) -> RwLockWriteGuard<'_, BTreeMap<Handle, Entry>> {
        todo!("source-profiler port")
    }

    /// The profiler collects (`sp_enabled`).
    pub fn is_enabled(&self) -> bool {
        todo!("source-profiler port")
    }

    /// GPU timing is on (`gpu_enabled`).
    pub fn is_gpu_enabled(&self) -> bool {
        todo!("source-profiler port")
    }

    /// Ring capacity of new entries: five seconds of frames.
    pub fn samples_per_entry(&self) -> u64 {
        todo!("source-profiler port")
    }

    /// `source_profiler_enable`: applied at the next frame.
    pub fn enable(&self, enable: bool) {
        todo!("source-profiler port")
    }

    /// `source_profiler_gpu_enable`: applied at the next frame, and only
    /// while an enable is pending or applied.
    pub fn gpu_enable(&self, enable: bool) {
        todo!("source-profiler port")
    }

    /// `source_profiler_reset_video`: five seconds of frames at
    /// `ceil(fps_num / fps_den)` fps, and everything dropped. The video
    /// thread is not running.
    pub fn reset_video<H: Host>(&self, fps_num: u32, fps_den: u32, host: &H) {
        todo!("source-profiler port")
    }

    /// `profiler_shutdown`: drop every sample and entry, destroy their
    /// timers and the timer ranges.
    fn shutdown<H: Host>(&self, host: &H) {
        todo!("source-profiler port")
    }

    /// `reset_gpu_timers`.
    fn reset_gpu_timers<H: Host>(&self, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_render_begin`: starts this frame's GPU timer range.
    pub fn render_begin<H: Host>(&self, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_render_end`.
    pub fn render_end<H: Host>(&self, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_frame_begin`: applies a pending enable, and
    /// advances the timer range while GPU timing runs.
    pub fn frame_begin(&self) {
        todo!("source-profiler port")
    }

    /// `source_profiler_frame_collect`: pushes each source's sample of the
    /// previous frame into its entry, then applies a pending disable.
    pub fn frame_collect<H: Host>(&self, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_async_frame_received`.
    pub fn async_frame_received<H: Host>(&self, source: Handle, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_source_tick_start`: the start time, 0 when off.
    pub fn source_tick_start<H: Host>(&self, host: &H) -> u64 {
        todo!("source-profiler port")
    }

    /// `source_profiler_source_tick_end`: a source's first tick creates its
    /// samples; later ones advance its frame first (a tick happens once, at
    /// the start of each frame).
    pub fn source_tick_end<H: Host>(&self, source: Handle, start: u64, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_source_render_begin`: `None` when off (the C
    /// returns 0 and leaves `*timer` alone), else the start time and the
    /// started GPU timer (0 without GPU timing).
    pub fn source_render_begin<H: Host>(&self, host: &H) -> Option<(u64, Handle)> {
        todo!("source-profiler port")
    }

    /// `source_profiler_source_render_end`: records a render pass in the
    /// source's current frame. A timer of a source without samples is
    /// destroyed.
    pub fn source_render_end<H: Host>(&self, source: Handle, start: u64, timer: Handle, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_remove_source`: whether to queue
    /// [`SourceProfiler::delete_source`] on the graphics thread.
    pub fn should_queue_removal(&self) -> bool {
        todo!("source-profiler port")
    }

    /// `task_delete_source`: drops a source's samples and entry.
    pub fn delete_source<H: Host>(&self, source: Handle, host: &H) {
        todo!("source-profiler port")
    }

    /// `source_profiler_fill_result`.
    pub fn fill_result<H: Host>(&self, source: Handle, host: &H) -> Fill {
        todo!("source-profiler port")
    }

    /// A copy of a source's entry, for tests and the differential check.
    pub fn entry(&self, source: Handle) -> Option<Entry> {
        todo!("source-profiler port")
    }

    /// A copy of a source's samples, for tests and the differential check.
    pub fn samples(&self, source: Handle) -> Option<SourceSamples> {
        todo!("source-profiler port")
    }

    /// The sources with samples, in key order.
    pub fn sampled_sources(&self) -> Vec<Handle> {
        todo!("source-profiler port")
    }

    /// The sources with entries, in key order.
    pub fn entry_sources(&self) -> Vec<Handle> {
        todo!("source-profiler port")
    }

    /// The current timer range slot and the ranges (`timer_idx`,
    /// `timer_ranges`).
    pub fn timer_ranges(&self) -> (usize, [Handle; FRAME_BUFFER_SIZE]) {
        todo!("source-profiler port")
    }
}

/// `frame_sample_destroy`'s timers, in one graphics context.
fn destroy_timers<H: Host>(timers: &[Handle], host: &H) {
    todo!("source-profiler port")
}

/// One source's part of `source_profiler_frame_collect`, without the async
/// timestamp.
fn collect_sample<H: Host>(
    entry: &mut Entry,
    sample: &mut FrameSample,
    gpu_valid: bool,
    frequency: u64,
    host: &H,
) {
    todo!("source-profiler port")
}

/// `util_mul_div64(ticks, 1000000000, frequency)`. A frequency of 0 gives
/// 0; the C divides by zero.
pub fn ticks_to_ns(ticks: u64, frequency: u64) -> u64 {
    todo!("source-profiler port")
}
