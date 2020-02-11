//! A safe API for creating HexChat plugins.
//!
//! To get started, create a struct representing your plugin, and implement `Plugin` for it. Then,
//! call `plugin!` on the struct.
//!
//! All plugins should be built as cdylibs, or if for some reason you have no other choice, dylibs.
//! Do not attempt to define a `main()` symbol; `Plugin::new` is your plugin's 'entry point'. For that
//! matter, do not attempt to define the HexChat C docs' described `extern fn`s - this is taken care
//! of for you by the `plugin!` macro.
//!
//! If window manipulation is desired, the `window` feature should be enabled.
//!
//! Static variables holding heap resources are discouraged and will cause memory leaks. This crate
//! provides a `safe_static!` macro for this purpose. Please note that any thread that you create
//! that accesses a safe static must be killed in your plugin's `Drop` implementation, and it's
//! undefined not to. You should kill them anyway even if you don't use this, because they'll be a
//! memory leak too otherwise.

#![deny(missing_docs, clippy::pedantic)]
#![allow(
    stable_features,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions
)]
#![feature(
    slice_patterns,
    trait_alias,
    type_alias_enum_variants,
    never_type,
    proc_macro_hygiene,
    try_from
)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[doc(hidden)]
pub mod c;

mod get_info;
pub use crate::get_info::*;
mod lists;
pub use crate::lists::*;
mod hook;
pub use crate::hook::*;
mod other;
pub use crate::other::*;
mod msg;
pub use crate::msg::*;
mod prefs;
pub use crate::prefs::*;
mod chan;
pub use crate::chan::*;
mod subplugin;
pub use crate::subplugin::*;
mod mask;
pub use crate::mask::*;
#[macro_use]
mod safe_static;
pub use crate::safe_static::*;

/// Server events for use with `add_server_event_listener`.
pub mod server_event;

/// Server responses for use with `add_server_response_listener`.
pub mod reply;

#[macro_use]
#[doc(hidden)]
pub mod call;

fn to_cstring(str: &str) -> CString {
    CString::new(str).unwrap_or_else(|_| CString::new(str.replace('\0', "")).unwrap())
}

unsafe fn from_cstring(ptr: *const c_char) -> String {
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

unsafe fn from_cstring_opt(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(from_cstring(ptr))
    }
}

/// Executes a command as though it were typed in HexChat's input box.
pub fn send_command(command: &str) {
    let command = to_cstring(command);
    unsafe {
        c!(hexchat_command, command.as_ptr());
    }
}

/// This trait must be implemented on a type before the type is passed to `plugin!`.
pub trait Plugin {
    /// The name of your plugin.
    const NAME: &'static str;
    /// A short description of your plugin.
    const DESC: &'static str = "";
    /// The version string of your plugin.
    const VERSION: &'static str = "";
    /// Creates a new instance of your plugin. This is your 'entry point'.
    fn new() -> Self;
}
