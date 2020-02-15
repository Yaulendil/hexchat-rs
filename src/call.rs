/// This macro will set up your plugin's actual callability from HexChat.
///
/// # Example
///
/// ```rust
/// use hexchat::{Plugin, plugin}
/// struct MyPlugin;
/// impl Plugin for MyPlugin {
///     //...
/// #    const NAME: &'static str = "myplugin";
/// #    fn new() -> Self {
/// #        Self
/// #    }
/// }
/// plugin!(MyPlugin);
/// ```
///
#[macro_export]
macro_rules! plugin {
    ($x:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn hexchat_plugin_init(
            plugin_handle: *mut $crate::c::hexchat_plugin,
            plugin_name: *mut *const ::std::os::raw::c_char,
            plugin_desc: *mut *const ::std::os::raw::c_char,
            plugin_version: *mut *const ::std::os::raw::c_char,
            arg: *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int {
            $crate::call::hexchat_plugin_init::<$x>(
                plugin_handle,
                plugin_name,
                plugin_desc,
                plugin_version,
                arg,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn hexchat_plugin_deinit(
            plugin_handle: *mut $crate::c::hexchat_plugin,
        ) -> ::std::os::raw::c_int {
            $crate::call::hexchat_plugin_deinit::<$x>(plugin_handle)
        }
    };
}

use parking_lot::{MappedRwLockWriteGuard, RwLock, RwLockWriteGuard};
use std::any::Any;
use std::collections::HashSet;
use std::mem;
use std::os::raw::{c_char, c_int};
use std::panic;

use crate::{
    c, to_cstring, Command, Plugin, PrintEventListener, RawServerEventListener,
    ServerEventListener, TimerTask, WindowEventListener, ALLOCATED, EXITING,
};
use std::sync::atomic::Ordering;

static PLUGIN: RwLock<Option<PluginDef>> = RwLock::new(None);
static PLUGIN_INSTANCE: RwLock<Option<PluginInstance>> = RwLock::new(None);
unsafe impl Sync for PluginDef {}
unsafe impl Send for PluginDef {}

pub(crate) fn get_plugin() -> MappedRwLockWriteGuard<'static, PluginDef> {
    RwLockWriteGuard::map(PLUGIN.write(), |o| o.as_mut().unwrap())
}

pub(crate) fn get_handle() -> *mut c::hexchat_plugin {
    get_plugin().ph
}

struct PluginInstance(Box<dyn Any>);
unsafe impl Send for PluginInstance {}
unsafe impl Sync for PluginInstance {}

#[allow(dead_code)]
pub(crate) fn is_loaded() -> bool {
    PLUGIN.read().is_some()
}

pub(crate) struct PluginDef {
    pub(crate) ph: *mut c::hexchat_plugin,
    pub(crate) commands: HashSet<Command>,
    pub(crate) print_events: HashSet<PrintEventListener>,
    pub(crate) window_events: HashSet<WindowEventListener>,
    pub(crate) server_events: HashSet<RawServerEventListener>,
    pub(crate) timer_tasks: HashSet<TimerTask>,
    pub(crate) typed_server_events: HashSet<ServerEventListener>,
}

pub unsafe fn hexchat_plugin_init<T>(
    plugin_handle: *mut c::hexchat_plugin,
    plugin_name: *mut *const c_char,
    plugin_desc: *mut *const c_char,
    plugin_version: *mut *const c_char,
    _arg: *mut c_char,
) -> c_int
where
    T: Plugin + 'static,
{
    {
        *ALLOCATED.write() = Some(Vec::new());
        let plugin_def = PluginDef {
            commands: HashSet::new(),
            print_events: HashSet::new(),
            window_events: HashSet::new(),
            server_events: HashSet::new(),
            timer_tasks: HashSet::new(),
            typed_server_events: HashSet::new(),
            ph: plugin_handle,
        };
        *PLUGIN.write() = Some(plugin_def);
    }
    let name = to_cstring(T::NAME);
    *plugin_name = name.into_raw();
    let desc = to_cstring(T::DESC);
    *plugin_desc = desc.into_raw();
    let version = to_cstring(T::VERSION);
    *plugin_version = version.into_raw();
    let t = panic::catch_unwind(T::new);
    let t = match t {
        Ok(t) => t,
        Err(e) => {
            crate::send_command(&if let Some(string) = (*e).downcast_ref::<&str>() {
                format!(
                    r#"GUI MSGBOX "Plugin '{} {}' failed to load. Panic message: {}""#,
                    T::NAME,
                    T::VERSION,
                    string
                )
            } else if let Some(string) = (*e).downcast_ref::<String>() {
                format!(
                    r#"GUI MSGBOX "Plugin '{} {}' failed to load. Panic message: {}""#,
                    T::NAME,
                    T::VERSION,
                    string
                )
            } else {
                format!(
                    r#"GUI MSGBOX "Plugin '{} {}' failed to load.""#,
                    T::NAME,
                    T::VERSION
                )
            });
            return -4;
        }
    };
    *PLUGIN_INSTANCE.write() = Some(PluginInstance(Box::new(t)));
    1
}

pub unsafe fn hexchat_plugin_deinit<T>(_plugin_handle: *mut c::hexchat_plugin) -> c_int
where
    T: Plugin,
{
    let mut instance = None;
    let mut plugin_write = PLUGIN_INSTANCE.write();

    //  Take the Value out of the Lock and immediately release it.
    mem::swap(&mut instance, &mut *plugin_write);
    mem::drop(plugin_write);
    if let Some(i) = instance { mem::drop(i); } else { return -2; }

    let mut plugin = None;
    let mut write = PLUGIN.write();

    //  Take the Value out of the Lock and immediately release it.
    mem::swap(&mut plugin, &mut *write);
    mem::drop(write);
    let PluginDef {
        server_events,
        window_events,
        print_events,
        commands,
        timer_tasks,
        typed_server_events,
        ..
    } = match plugin {
        Some(p) => p,
        None => return 1,
    };

    EXITING.store(false, Ordering::SeqCst);
    for event in server_events {
        crate::dealloc_raw_server_event_listener(event.0);
    }
    for event in window_events {
        crate::dealloc_window_event_listener(event.0);
    }
    for event in print_events {
        crate::dealloc_print_event_listener(event.0);
    }
    for command in commands {
        crate::dealloc_command(command.0);
    }
    for timer_task in timer_tasks {
        crate::dealloc_timer_task(timer_task.0);
    }
    for event in typed_server_events {
        crate::dealloc_server_event_listener(event.0);
    }
    let mut vec = None;
    let mut lock = ALLOCATED.write();
    mem::swap(&mut vec, &mut *lock);
    if let Some(vec) = vec {
        for func in vec {
            let boxed = func.0;
            boxed();
        }
    } else {
        return -5;
    }
    1
}
