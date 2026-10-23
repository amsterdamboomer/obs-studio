//! Port of `libobs/obs-view-api.c.h` (#1933): a view's channel table, the
//! activation of the sources it holds, and its video mixes.
//!
//! `struct obs_view` stays C, embedded in `struct obs_canvas`; so do the
//! sources and the video mixes list. The core reaches all of them through a
//! [`Host`], one step per call, so every lock, reference and activation
//! happens in the C's order. Handles are the C pointers as addresses; 0 is
//! NULL.
//!
//! The C asserts `channel < MAX_CHANNELS` before it checks the view; release
//! builds then return. The port has no assert: an out-of-range channel
//! returns as the release C does.

/// A C pointer as an address; 0 is NULL.
pub type Handle = usize;

/// `MAX_CHANNELS` in `obs-defs.h`.
pub const MAX_CHANNELS: usize = 64;

/// `enum view_type` in `obs-internal.h`.
pub const MAIN_VIEW: u32 = 1;
/// `enum view_type` in `obs-internal.h`.
pub const AUX_VIEW: u32 = 2;

/// The C steps a view takes. libobs implements them in `obs-view.c` and
/// `obs-source.c`; the tests record them.
pub trait Host {
    /// `bzalloc(sizeof(struct obs_view))`.
    fn alloc_view(&self) -> Handle;
    /// `bfree(view)`.
    fn free_view(&self, view: Handle);
    /// Initialises the channel mutex; false when `pthread_mutex_init` fails.
    fn mutex_init(&self, view: Handle) -> bool;
    fn mutex_destroy(&self, view: Handle);
    fn lock(&self, view: Handle);
    fn unlock(&self, view: Handle);
    fn view_type(&self, view: Handle) -> u32;
    fn set_view_type(&self, view: Handle, view_type: u32);
    fn channel(&self, view: Handle, channel: usize) -> Handle;
    fn set_channel(&self, view: Handle, channel: usize, source: Handle);
    /// `memset(view->channels, 0, ...)`.
    fn clear_channels(&self, view: Handle);
    /// `blog(LOG_ERROR, "obs_view_init: Failed to create mutex")`.
    fn log_mutex_failed(&self);

    /// `obs_source_get_ref`: NULL for NULL or a source being destroyed.
    fn source_get_ref(&self, source: Handle) -> Handle;
    fn source_release(&self, source: Handle);
    fn source_activate(&self, source: Handle, view_type: u32);
    fn source_deactivate(&self, source: Handle, view_type: u32);
    fn source_video_render(&self, source: Handle);
    /// `source->removed`.
    fn source_removed(&self, source: Handle) -> bool;

    /// The main canvas mix's `obs_video_info`, or 0 with no mix.
    fn main_mix_info(&self) -> Handle;
    /// Creates a mix for the view and lists it; its `video_t`, or 0.
    fn mix_add(&self, view: Handle, ovi: Handle) -> Handle;
    fn mixes_lock(&self);
    fn mixes_unlock(&self);
    fn mix_count(&self) -> usize;
    fn mix_view(&self, index: usize) -> Handle;
    fn mix_clear_view(&self, index: usize);
    fn mix_info(&self, index: usize) -> Handle;
}

/// `obs_view_init`: the mutex, then the type. False for a NULL view or a
/// failed mutex (logged).
pub fn init<H: Host>(host: &H, view: Handle, view_type: u32) -> bool {
    todo!("#1933")
}

/// `obs_view_create`: an aux view, or 0 when its mutex fails.
pub fn create<H: Host>(host: &H) -> Handle {
    todo!("#1933")
}

/// `obs_view_free`: deactivates and releases every channel's source, then
/// clears the channels and destroys the mutex. The view is not freed.
pub fn free<H: Host>(host: &H, view: Handle) {
    todo!("#1933")
}

/// `obs_view_destroy`: [`free`], then the allocation.
pub fn destroy<H: Host>(host: &H, view: Handle) {
    todo!("#1933")
}

/// `obs_view_get_source`: a new reference to the channel's source.
pub fn get_source<H: Host>(host: &H, view: Handle, channel: u32) -> Handle {
    todo!("#1933")
}

/// `obs_view_set_source`: takes a reference to `source` and swaps it in
/// under the lock; after it, activates the new source and deactivates and
/// releases the old one.
pub fn set_source<H: Host>(host: &H, view: Handle, channel: u32, source: Handle) {
    todo!("#1933")
}

/// `obs_view_render`: under the lock, drops (releases, without
/// deactivating) each removed source and renders the others.
pub fn render<H: Host>(host: &H, view: Handle) {
    todo!("#1933")
}

/// `obs_view_add`: [`add2`] with the main canvas mix's video info.
pub fn add<H: Host>(host: &H, view: Handle) -> Handle {
    todo!("#1933")
}

/// `obs_view_add2`: a new mix for the view.
pub fn add2<H: Host>(host: &H, view: Handle, ovi: Handle) -> Handle {
    todo!("#1933")
}

/// `obs_view_remove`: detaches the view from every mix that renders it.
pub fn remove<H: Host>(host: &H, view: Handle) {
    todo!("#1933")
}

/// `obs_view_enum_video_info`: the video info of each of the view's mixes,
/// until `f` returns false. The mix count is read once, as the C does.
pub fn enum_video_info<H: Host>(host: &H, view: Handle, f: impl FnMut(Handle) -> bool) {
    todo!("#1933")
}
