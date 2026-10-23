//! Port of `libobs/graphics/image-file.c.h`: `gs_image_file_ex_*`.
//!
//! An image file loaded for a texture. A `.gif` with more than one frame is
//! decoded by libnsgif and animated by elapsed time; anything else, a
//! one-frame GIF included, is decoded by `gs_create_texture_file_data3`
//! (graphics-ffmpeg). [`ImageFileEx`] is `struct gs_image_file_ex` from the
//! public `graphics/image-file.h`, generic over the handles a [`Host`]
//! hands out: the C ABI shim uses the real pointers, tests use fakes.
//!
//! Everything outside the file stays where it is and is reached through
//! [`Host`]: libnsgif, the file read, `bmalloc`/`bzalloc`/`bfree`, the
//! graphics calls and the log. The host calls are made in the C order, with
//! the C arguments, so a recording host sees the same sequence.
//!
//! Characterized, not endorsed (kept as the C does it):
//! - A `.gif` that cannot be opened or read, cannot be scanned, or is larger
//!   than 4096 in either direction fails without trying FFmpeg. Its nsgif
//!   object is not destroyed.
//! - Init never fills `mem_usage`, `alpha_mode` or `space`.
//! - Only frame 0 gets the alpha mode given to init; frames decoded later by
//!   tick and update are straight alpha.
//! - A GIF that loops forever never counts `cur_loop` up.
//! - A tick that comes back to the current frame returns false and decodes
//!   nothing.
//! - A frame delay of 0 plays for 100 ms.
//! - Scanning repeats until libnsgif reports success, so a scan error that
//!   persists does not end the loop.

use core::ffi::CStr;

use crate::texture_render::{ColorFormat, ColorSpace};

/// `enum gs_image_alpha_mode`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageAlphaMode(pub u32);

impl ImageAlphaMode {
    pub const STRAIGHT: Self = Self(0);
    pub const PREMULTIPLY_SRGB: Self = Self(1);
    pub const PREMULTIPLY: Self = Self(2);
}

/// `GS_RGBA`.
pub const GS_RGBA: ColorFormat = ColorFormat(3);
/// `GS_DYNAMIC`, the flag of an animated GIF's texture.
pub const GS_DYNAMIC: u32 = 1 << 1;
/// `NSGIF_OK`.
pub const NSGIF_OK: u32 = 0;
/// `LOG_WARNING`.
pub const LOG_WARNING: i32 = 200;

/// The `nsgif_info_t` fields the file reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GifInfo {
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub loop_max: i32,
}

/// What reading the whole file gave, in the C steps: open, size, allocate,
/// read.
pub enum FileRead<D> {
    /// `os_fopen` failed; nothing was allocated.
    OpenFailed,
    /// The file's size and the `bmalloc` block of that size; `complete` is
    /// whether `fread` filled it.
    Read {
        data: D,
        size: usize,
        complete: bool,
    },
}

/// The calls the file makes outside itself. Handles are plain values the
/// host owns; the core never looks inside them.
pub trait Host: Sized {
    /// `gs_texture_t *`.
    type Texture: Copy;
    /// A `bmalloc`/`bzalloc` byte block.
    type Data: Copy;
    /// The `uint8_t **` frame cache: one pointer per frame into the frame
    /// data, null until that frame is decoded.
    type Cache: Copy;
    /// `nsgif_t *`.
    type Gif: Copy;
    /// `nsgif_bitmap_t *`: libnsgif's current frame, `4 * width * height`
    /// RGBA bytes.
    type Bitmap: Copy;
    /// `struct gs_image_file_internal *`, a `bzalloc` block.
    type Internal;

    /// `bzalloc(sizeof(struct gs_image_file_internal))`.
    fn internal_new() -> Self::Internal;
    /// The block behind an internal handle.
    fn internal(handle: &mut Self::Internal) -> &mut Internal<Self>;
    /// `bfree` of the internal block.
    fn internal_free(handle: Self::Internal);

    /// `nsgif_create` with the file's bitmap callbacks, RGBA.
    fn gif_create() -> Option<Self::Gif>;
    /// `os_fopen` .. `fread` of the whole file.
    fn read_file(path: &CStr) -> FileRead<Self::Data>;
    /// `nsgif_data_scan`.
    fn gif_scan(gif: Option<Self::Gif>, size: usize, data: Option<Self::Data>) -> u32;
    /// `nsgif_data_complete`.
    fn gif_complete(gif: Option<Self::Gif>);
    /// `*nsgif_get_info`.
    fn gif_info(gif: Option<Self::Gif>) -> GifInfo;
    /// `nsgif_frame_decode`: the error, and the bitmap when it is `NSGIF_OK`.
    fn gif_decode(gif: Option<Self::Gif>, frame: u32) -> (u32, Option<Self::Bitmap>);
    /// `nsgif_get_frame_info(gif, frame)->delay`, in centiseconds.
    fn gif_frame_delay(gif: Option<Self::Gif>, frame: i32) -> u32;
    /// `nsgif_destroy`.
    fn gif_destroy(gif: Option<Self::Gif>);
    /// Runs `f` over the `len` bytes of a bitmap.
    fn with_bitmap(bitmap: Self::Bitmap, len: usize, f: &mut dyn FnMut(&mut [u8]));

    /// `bzalloc(size)`.
    fn alloc_data(size: usize) -> Self::Data;
    /// `bzalloc(count * sizeof(uint8_t *))`.
    fn alloc_cache(count: u32) -> Self::Cache;
    /// `bfree`.
    fn free_data(data: Option<Self::Data>);
    /// `bfree`.
    fn free_cache(cache: Option<Self::Cache>);
    /// Whether `cache[frame]` is set.
    fn cache_has(cache: Option<Self::Cache>, frame: i32) -> bool;
    /// `cache[frame] = frame_data + offset`, then the `len` bitmap bytes
    /// copied there.
    fn cache_store(
        cache: Option<Self::Cache>,
        frame: i32,
        frame_data: Option<Self::Data>,
        offset: usize,
        bitmap: Option<Self::Bitmap>,
        len: usize,
    );

    /// `gs_create_texture_file_data3`, which writes the outputs only on
    /// success.
    fn decode_file(
        path: &CStr,
        alpha_mode: ImageAlphaMode,
        format: &mut ColorFormat,
        cx: &mut u32,
        cy: &mut u32,
        space: &mut ColorSpace,
    ) -> Option<Self::Data>;
    /// `gs_texture_create(cx, cy, format, 1, &bitmap, GS_DYNAMIC)`.
    fn texture_create_bitmap(
        cx: u32,
        cy: u32,
        format: ColorFormat,
        bitmap: Option<Self::Bitmap>,
    ) -> Option<Self::Texture>;
    /// `gs_texture_create(cx, cy, format, 1, &data, 0)`.
    fn texture_create_data(
        cx: u32,
        cy: u32,
        format: ColorFormat,
        data: Option<Self::Data>,
    ) -> Option<Self::Texture>;
    /// `gs_texture_destroy`, also with null.
    fn texture_destroy(texture: Option<Self::Texture>);
    /// `gs_texture_set_image(texture, cache[frame], linesize, false)`.
    fn texture_set_image(
        texture: Option<Self::Texture>,
        cache: Option<Self::Cache>,
        frame: i32,
        linesize: u32,
    );

    /// `blog(level, ...)` of the formatted line, raw bytes as `%s` prints.
    fn log(level: i32, message: &[u8]);
}

/// `struct gs_image_file_internal`, private to the file.
pub struct Internal<H: Host> {
    pub gif: Option<H::Gif>,
    pub gif_info: GifInfo,
    pub gif_frame_image: Option<H::Bitmap>,
    pub gif_data: Option<H::Data>,
    pub animation_frame_cache: Option<H::Cache>,
    pub animation_frame_data: Option<H::Data>,
    pub last_decoded_frame: i32,
}

impl<H: Host> Default for Internal<H> {
    fn default() -> Self {
        Self {
            gif: None,
            gif_info: GifInfo::default(),
            gif_frame_image: None,
            gif_data: None,
            animation_frame_cache: None,
            animation_frame_data: None,
            last_decoded_frame: 0,
        }
    }
}

/// `struct gs_image_file_ex`, field for field.
#[repr(C)]
pub struct ImageFileEx<H: Host> {
    pub texture: Option<H::Texture>,
    pub format: ColorFormat,
    pub cx: u32,
    pub cy: u32,
    pub is_animated_gif: bool,
    pub frame_updated: bool,
    pub loaded: bool,
    pub internal: Option<H::Internal>,
    pub cur_time: u64,
    pub cur_frame: i32,
    pub cur_loop: i32,
    pub texture_data: Option<H::Data>,
    pub mem_usage: u64,
    pub alpha_mode: ImageAlphaMode,
    pub space: ColorSpace,
}

impl<H: Host> Default for ImageFileEx<H> {
    fn default() -> Self {
        Self {
            texture: None,
            format: ColorFormat(0),
            cx: 0,
            cy: 0,
            is_animated_gif: false,
            frame_updated: false,
            loaded: false,
            internal: None,
            cur_time: 0,
            cur_frame: 0,
            cur_loop: 0,
            texture_data: None,
            mem_usage: 0,
            alpha_mode: ImageAlphaMode::STRAIGHT,
            space: ColorSpace(0),
        }
    }
}

/// `gs_get_format_bpp`.
pub fn format_bpp(_format: ColorFormat) -> u32 {
    todo!()
}

/// `gs_premultiply_xyza_loop` (`srgb` false) and
/// `gs_premultiply_xyza_srgb_loop` (`srgb` true) over whole texels.
pub fn premultiply(_data: &mut [u8], _srgb: bool) {
    todo!()
}

/// Whether the path ends in `.gif`, any case, with something before it.
pub fn is_gif_path(_path: &[u8]) -> bool {
    todo!()
}

/// `get_time`: a frame delay in centiseconds as nanoseconds, 0 as 100 ms.
pub fn frame_time_ns(_delay: u32) -> u64 {
    todo!()
}

/// The loop limit tick uses: libnsgif's `loop_max`, 0 (forever) from 0xFFFF.
pub fn effective_loops(_loop_max: i32) -> i32 {
    todo!()
}

impl<H: Host> ImageFileEx<H> {
    /// `gs_image_file_ex_init`.
    pub fn init(&mut self, _file: Option<&CStr>, _alpha_mode: ImageAlphaMode) {
        todo!()
    }

    /// `gs_image_file_ex_free`.
    pub fn free(&mut self) {
        todo!()
    }

    /// `gs_image_file_ex_init_texture`.
    pub fn init_texture(&mut self) {
        todo!()
    }

    /// `gs_image_file_ex_tick`.
    pub fn tick(&mut self, _elapsed_time_ns: u64) -> bool {
        todo!()
    }

    /// `gs_image_file_ex_update_texture`.
    pub fn update_texture(&mut self) {
        todo!()
    }
}
