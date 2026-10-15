//! Safe core for `libobs/obs-properties.c`.
//!
//! A properties set ([`PropsRc`]) owns its properties ([`PropRc`]) in
//! insertion order, and a group property owns its content set. Parents are
//! `Weak` links, so there are no cycles. Callbacks are stored here as data;
//! the C ABI layer calls them, never while a `RefCell` is borrowed, because
//! plugin callbacks look up, change, add and remove properties.
//!
//! Enum-typed fields hold the raw C values: C stores whatever `int` the
//! caller passes, and the getters return it unchanged.

#![allow(
    unused_variables,
    unused_imports,
    unused_mut,
    dead_code,
    clippy::ptr_arg
)] // RED stubs (#828)

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_void};
use std::rc::{Rc, Weak};

pub type PropsRc = Rc<RefCell<Props>>;
pub type PropRc = Rc<RefCell<Prop>>;

/// `obs_property_modified_t`.
pub type ModifiedFn =
    unsafe extern "C" fn(props: *mut c_void, p: *mut c_void, settings: *mut c_void) -> bool;
/// `obs_property_modified2_t`.
pub type Modified2Fn = unsafe extern "C" fn(
    priv_: *mut c_void,
    props: *mut c_void,
    p: *mut c_void,
    settings: *mut c_void,
) -> bool;
/// `obs_property_clicked_t`.
pub type ClickedFn =
    unsafe extern "C" fn(props: *mut c_void, p: *mut c_void, data: *mut c_void) -> bool;
/// The `destroy` callback of `obs_properties_set_param`.
pub type DestroyFn = unsafe extern "C" fn(param: *mut c_void);

/// A param and its destroy callback, to be called by the C ABI layer.
pub type PendingDestroy = (usize, DestroyFn);

pub mod ty {
    pub const INVALID: i32 = 0;
    pub const BOOL: i32 = 1;
    pub const INT: i32 = 2;
    pub const FLOAT: i32 = 3;
    pub const TEXT: i32 = 4;
    pub const PATH: i32 = 5;
    pub const LIST: i32 = 6;
    pub const COLOR: i32 = 7;
    pub const BUTTON: i32 = 8;
    pub const FONT: i32 = 9;
    pub const EDITABLE_LIST: i32 = 10;
    pub const FRAME_RATE: i32 = 11;
    pub const GROUP: i32 = 12;
    pub const COLOR_ALPHA: i32 = 13;
}

pub const COMBO_FORMAT_INVALID: i32 = 0;
pub const COMBO_FORMAT_INT: i32 = 1;
pub const COMBO_FORMAT_FLOAT: i32 = 2;
pub const COMBO_FORMAT_STRING: i32 = 3;
pub const COMBO_FORMAT_BOOL: i32 = 4;
pub const COMBO_TYPE_INVALID: i32 = 0;
pub const COMBO_TYPE_EDITABLE: i32 = 1;
pub const PATH_FILE: i32 = 0;
pub const PATH_FILE_SAVE: i32 = 1;
pub const PATH_DIRECTORY: i32 = 2;
pub const TEXT_DEFAULT: i32 = 0;
pub const TEXT_INFO_NORMAL: i32 = 0;
pub const NUMBER_SCROLLER: i32 = 0;
pub const NUMBER_SLIDER: i32 = 1;
pub const EDITABLE_LIST_TYPE_STRINGS: i32 = 0;
/// `obs_property_group_type`'s value for a non-group: C returns
/// `OBS_COMBO_INVALID`, the list enum's 0.
pub const COMBO_INVALID: i32 = 0;
pub const BUTTON_DEFAULT: i32 = 0;

/// `struct media_frames_per_second`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fps {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Default)]
pub struct Number<T> {
    pub min: T,
    pub max: T,
    pub step: T,
    pub kind: i32,
    pub suffix: Option<CString>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(Option<CString>),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug)]
pub struct ListItem {
    pub name: Option<CString>,
    pub disabled: bool,
    pub value: Value,
}

#[derive(Debug)]
pub enum Data {
    None,
    Int(Number<i32>),
    Float(Number<f64>),
    Text {
        kind: i32,
        monospace: bool,
        info_kind: i32,
        info_word_wrap: bool,
    },
    Path {
        kind: i32,
        filter: Option<CString>,
        default_path: Option<CString>,
    },
    List {
        items: Vec<ListItem>,
        kind: i32,
        format: i32,
    },
    EditableList {
        kind: i32,
        filter: Option<CString>,
        default_path: Option<CString>,
    },
    Button {
        callback: Option<ClickedFn>,
        kind: i32,
        url: Option<CString>,
    },
    FrameRate {
        options: Vec<(Option<CString>, Option<CString>)>,
        ranges: Vec<(Fps, Fps)>,
    },
    Group {
        kind: i32,
        content: PropsRc,
    },
}

#[derive(Debug)]
pub struct Props {
    pub param: usize,
    pub destroy: Option<DestroyFn>,
    pub flags: u32,
    pub items: Vec<PropRc>,
    /// The group property whose content this set is.
    pub parent: Weak<RefCell<Prop>>,
}

#[derive(Debug)]
pub struct Prop {
    pub name: CString,
    pub desc: Option<CString>,
    pub long_desc: Option<CString>,
    pub priv_: usize,
    pub kind: i32,
    pub visible: bool,
    pub enabled: bool,
    pub parent: Weak<RefCell<Props>>,
    pub modified: Option<ModifiedFn>,
    pub modified2: Option<Modified2Fn>,
    pub data: Data,
}

/// `bstrdup`: NULL stays NULL, anything else (also "") is copied.
pub fn dup(s: Option<&CStr>) -> Option<CString> {
    todo!("obs-properties port (#828)")
}

/// `s && *s ? bstrdup(s) : NULL`.
pub fn dup_nonempty(s: Option<&CStr>) -> Option<CString> {
    todo!("obs-properties port (#828)")
}

/* ------------------------------------------------------------------------- */
/* sets */

pub fn create() -> PropsRc {
    todo!("obs-properties port (#828)")
}

/// `obs_properties_set_param`: returns the old param to destroy, if C would.
pub fn set_param(
    props: &PropsRc,
    param: usize,
    destroy: Option<DestroyFn>,
) -> Option<PendingDestroy> {
    todo!("obs-properties port (#828)")
}

/// Everything `obs_properties_destroy` would destroy, in C's order: this
/// set's param, then each group's content, depth first.
pub fn destroy_list(props: &PropsRc) -> Vec<PendingDestroy> {
    todo!("obs-properties port (#828)")
}

pub fn first(props: &PropsRc) -> Option<PropRc> {
    todo!("obs-properties port (#828)")
}

/// `obs_property_next`: the following property of the same set.
pub fn next(prop: &PropRc) -> Option<PropRc> {
    todo!("obs-properties port (#828)")
}

/// `obs_properties_get`: this set first, then every group, recursively.
pub fn get(props: &PropsRc, name: &CStr) -> Option<PropRc> {
    todo!("obs-properties port (#828)")
}

/// `obs_properties_get_parent`: the set holding the group this set is the
/// content of.
pub fn parent(props: &PropsRc) -> Option<PropsRc> {
    todo!("obs-properties port (#828)")
}

/// `get_topmost_parent`.
pub fn topmost(props: &PropsRc) -> PropsRc {
    todo!("obs-properties port (#828)")
}

/// `has_prop`: the name is in use anywhere in the tree this set belongs to.
/// C logs `Property '%s' exists` when it is; the C ABI layer does that.
pub fn has_prop(props: &PropsRc, name: &CStr) -> bool {
    todo!("obs-properties port (#828)")
}

/// `obs_properties_remove_by_name`: removes the property from this set, or
/// else from every group, recursively. Returns what C would destroy.
pub fn remove_by_name(props: &PropsRc, name: &CStr) -> Vec<PendingDestroy> {
    todo!("obs-properties port (#828)")
}

/// The properties `obs_properties_apply_settings` calls back, in call
/// order: collected with each group's content before the group itself,
/// then called in reverse.
pub fn apply_settings_order(props: &PropsRc) -> Vec<PropRc> {
    todo!("obs-properties port (#828)")
}

/* ------------------------------------------------------------------------- */
/* adding */

/// Why an `obs_properties_add_*` call returned NULL.
#[derive(Debug, PartialEq, Eq)]
pub enum AddError {
    /// The name is in use in the tree (C logs `Property '%s' exists`).
    Exists,
    /// An editable combo with a non-string format (C logs it).
    EditableNotString,
    /// A NULL, self, recursive or clashing group (C does not log).
    BadGroup,
}

/// `new_prop` after the `has_prop` check.
pub fn add(
    props: &PropsRc,
    name: &CStr,
    desc: Option<&CStr>,
    kind: i32,
    data: Data,
) -> Result<PropRc, AddError> {
    todo!("obs-properties port (#828)")
}

pub fn number<T: Default>(min: T, max: T, step: T, kind: i32) -> Number<T> {
    todo!("obs-properties port (#828)")
}

pub fn add_list(
    props: &PropsRc,
    name: &CStr,
    desc: Option<&CStr>,
    kind: i32,
    format: i32,
) -> Result<PropRc, AddError> {
    todo!("obs-properties port (#828)")
}

/// `obs_properties_add_group`. `has_prop` runs for the name, then (C order)
/// for each content name while checking for duplicates; each hit is
/// reported through `on_exists` so the caller can log it.
pub fn add_group(
    props: &PropsRc,
    name: &CStr,
    desc: Option<&CStr>,
    kind: i32,
    group: Option<&PropsRc>,
    mut on_exists: impl FnMut(&CStr),
) -> Result<PropRc, AddError> {
    todo!("obs-properties port (#828)")
}

/* ------------------------------------------------------------------------- */
/* properties */

/// The set the property belongs to, for `get_topmost_parent(p->parent)`.
pub fn owner(prop: &PropRc) -> Option<PropsRc> {
    todo!("obs-properties port (#828)")
}

pub fn list_add(prop: &PropRc, format: i32, name: Option<&CStr>, value: Value) -> Option<usize> {
    todo!("obs-properties port (#828)")
}

/// `da_insert` asserts `idx <= num`; an index past the end is ignored here.
pub fn list_insert(prop: &PropRc, idx: usize, format: i32, name: Option<&CStr>, value: Value) {
    todo!("obs-properties port (#828)")
}

/// `da_insert_new` asserts `idx <= num`; an index past the end is ignored.
pub fn frame_rate_insert<T>(list: &mut Vec<T>, idx: usize, value: T) {
    todo!("obs-properties port (#828)")
}
