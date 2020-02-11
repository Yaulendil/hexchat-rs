#![allow(non_camel_case_types)]

use libc::time_t;
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct hexchat_context(!);
#[repr(C)]
pub struct hexchat_hook(!);
#[repr(C)]
pub struct hexchat_list(!);

#[repr(C)]
pub struct hexchat_event_attrs {
    pub server_time_utc: time_t,
    pub ircv3_line: *const c_char,
}

#[macro_export]
#[doc(hidden)]
macro_rules! c {
    ($name:ident, $($arg:expr),+) => {{
        let handle = $crate::call::get_handle();
        ((*handle).$name)(handle, $($arg),+)
    }};
    ($name:ident, $($arg:expr,)+) => {{
        let handle = $crate::call::get_handle();
        ((*handle).$name)(handle, $($arg),+)
    }};
    ($name:ident) => {{
        let handle = $crate::call::get_handle();
        ((*handle).$name)(handle)
    }};
}

#[repr(C)]
pub struct hexchat_plugin {
    pub hexchat_hook_command: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        pri: c_int,
        callback: unsafe extern "C" fn(
            word: *mut *mut c_char,
            word_eol: *mut *mut c_char,
            user_data: *mut c_void,
        ) -> c_int,
        help_text: *const c_char,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_hook_server: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        pri: c_int,
        callback: unsafe extern "C" fn(
            word: *mut *mut c_char,
            word_eol: *mut *mut c_char,
            user_data: *mut c_void,
        ) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_hook_print: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        pri: c_int,
        callback: unsafe extern "C" fn(word: *mut *mut c_char, user_data: *mut c_void) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_hook_timer: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        timeout: c_int,
        callback: unsafe extern "C" fn(user_data: *mut c_void) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_hook_fd: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        fd: c_int,
        flags: c_int,
        callback: unsafe extern "C" fn(fd_: c_int, flags_: c_int, user_data: *mut c_void) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_unhook:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, hook: *mut hexchat_hook) -> *mut c_void,

    pub hexchat_print: unsafe extern "C" fn(ph: *mut hexchat_plugin, text: *const c_char),

    pub hexchat_printf: unsafe extern "C" fn(ph: *mut hexchat_plugin, format: *const c_char, ...),

    pub hexchat_command: unsafe extern "C" fn(ph: *mut hexchat_plugin, command: *const c_char),

    pub hexchat_commandf: unsafe extern "C" fn(ph: *mut hexchat_plugin, format: *const c_char, ...),

    pub hexchat_nickcmp: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        s1: *const c_char,
        s2: *const c_char,
    ) -> c_int,

    pub hexchat_set_context:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, ctx: *mut hexchat_context) -> c_int,

    pub hexchat_find_context: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        servname: *const c_char,
        channel: *const c_char,
    ) -> *mut hexchat_context,

    pub hexchat_get_context: unsafe extern "C" fn(ph: *mut hexchat_plugin) -> *mut hexchat_context,

    pub hexchat_get_info:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, id: *const c_char) -> *const c_char,

    pub hexchat_get_prefs: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        string: *mut *const c_char,
        integer: *mut c_int,
    ) -> c_int,

    pub hexchat_list_get:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, name: *const c_char) -> *mut hexchat_list,

    pub hexchat_list_free: unsafe extern "C" fn(ph: *mut hexchat_plugin, xlist: *mut hexchat_list),

    pub hexchat_list_fields:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, name: *const c_char) -> *const *const c_char,

    pub hexchat_list_next:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, xlist: *mut hexchat_list) -> c_int,

    pub hexchat_list_str: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        xlist: *mut hexchat_list,
        name: *const c_char,
    ) -> *const c_char,

    pub hexchat_list_int: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        xlist: *mut hexchat_list,
        name: *const c_char,
    ) -> c_int,

    pub hexchat_plugingui_add: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        filename: *const c_char,
        name: *const c_char,
        desc: *const c_char,
        version: *const c_char,
        reserved: *const c_char,
    ) -> *mut c_void,

    pub hexchat_plugingui_remove:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, handle: *mut c_void),

    pub hexchat_emit_print:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, name: *const c_char, ...) -> c_int,

    pub hexchat_read_fd: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        src: *mut c_void,
        buf: *mut c_char,
        len: *mut c_int,
    ) -> c_int,

    pub hexchat_list_time: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        xlist: *mut hexchat_list,
        name: *const c_char,
    ) -> time_t,

    pub hexchat_gettext:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, msgid: *const c_char) -> *mut c_char,

    pub hexchat_send_modes: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        targets: *mut *const c_char,
        ntargets: c_int,
        modes_per_line: c_int,
        sign: c_char,
        mode: c_char,
    ),

    pub hexchat_strip: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        str: *const c_char,
        len: c_int,
        flags: c_int,
    ) -> *mut c_char,

    pub hexchat_free: unsafe extern "C" fn(ph: *mut hexchat_plugin, ptr: *mut c_void),

    pub hexchat_pluginpref_set_str: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        var: *const c_char,
        value: *const c_char,
    ) -> c_int,

    pub hexchat_pluginpref_get_str: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        var: *const c_char,
        dest: *mut c_char,
    ) -> c_int,

    pub hexchat_pluginpref_set_int:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, var: *const c_char, value: c_int) -> c_int,

    pub hexchat_pluginpref_get_int:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, var: *const c_char) -> c_int,

    pub hexchat_pluginpref_delete:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, var: *const c_char) -> c_int,

    pub hexchat_pluginpref_list:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, dest: *const c_char) -> c_int,

    pub hexchat_hook_server_attrs: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        pri: c_int,
        callback: unsafe extern "C" fn(
            word: *mut *mut c_char,
            word_eol: *mut *mut c_char,
            attrs: *mut hexchat_event_attrs,
            user_data: *mut c_void,
        ) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,
    pub hexchat_hook_print_attrs: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        name: *const c_char,
        pri: c_int,
        callback: unsafe extern "C" fn(
            word: *mut *mut c_char,
            attrs: *mut hexchat_event_attrs,
            user_data: *mut c_void,
        ) -> c_int,
        userdata: *mut c_void,
    ) -> *mut hexchat_hook,

    pub hexchat_emit_print_attrs: unsafe extern "C" fn(
        ph: *mut hexchat_plugin,
        attrs: *mut hexchat_event_attrs,
        event_name: *const c_char,
        ...
    ) -> c_int,

    pub hexchat_event_attrs_create:
        unsafe extern "C" fn(ph: *mut hexchat_plugin) -> *mut hexchat_event_attrs,

    pub hexchat_event_attrs_free:
        unsafe extern "C" fn(ph: *mut hexchat_plugin, attrs: *mut hexchat_event_attrs),
}
