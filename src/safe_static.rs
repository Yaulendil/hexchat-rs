use std::ops::Deref;

use parking_lot::{Once, OnceState, RwLock};
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __safe_static_internal {
    ($(#[$attr:meta])* ($($vis:tt)*) static lazy $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __safe_static_internal!(@LAZY TY, $(#[$attr])*, ($($vis)*), $N, $T, $e);
        safe_static!($($t)*);
    };
    ($(#[$attr:meta])* ($($vis:tt)*) static uninit $N:ident : $T:ty; $($t:tt)*) => {
        __safe_static_internal!(@UNINIT TY, $(#[$attr])*, ($($vis)*), $N, $T);
    };
    (@LAZY TY, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident, $T:ty, $e:expr) => {
        $(#[$attr])*
        $($vis)* static $N: $crate::SafeLazy<$T> = $crate::SafeLazy { instance: unsafe { $crate::SafeLazyInstance::new() }, init_fn: || { $e } };
    };
    (@UNINIT TY, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident, $T:ty) => {
        $(#[$attr])*
        $($vis)* static $N: $crate::SafeUninit<$T> = unsafe { $crate::SafeUninit::new() };
    };
    () => ()
}
/// A macro for creating `SafeLazy`s and `SafeUninit`s.
///
/// # Important
///
/// Any thread which accesses a safe static, mutex or no, must be killed inside your plugin's `Drop`
/// implementation. To allow otherwise is undefined.
#[macro_export(local_inner_macros)]
macro_rules! safe_static {
    ($(#[$attr:meta])* static lazy $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        // use `()` to explicitly forward the information about private items
        __safe_static_internal!($(#[$attr])* () static lazy $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static lazy $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __safe_static_internal!($(#[$attr])* (pub) static lazy $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static lazy $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        __safe_static_internal!($(#[$attr])* (pub ($($vis)+)) static lazy $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* static uninit $N:ident : $T:ty; $($t:tt)*) => {
        // use `()` to explicitly forward the information about private items
        __safe_static_internal!($(#[$attr])* () static uninit $N : $T; $($t)*);
    };
    ($(#[$attr:meta])* pub static uninit $N:ident : $T:ty; $($t:tt)*) => {
        __safe_static_internal!($(#[$attr])* (pub) static uninit $N : $T; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static uninit $N:ident : $T:ty; $($t:tt)*) => {
        __safe_static_internal!($(#[$attr])* (pub ($($vis)+)) static uninit $N : $T; $($t)*);
    };
    () => ()
}

/// A lazily-evaluated resource that is safe to use in a HexChat plugin.
///
/// It is recommended that you not use any other kind of static resource crate, because any static
/// resource will not get dropped when your plugin is unloaded, leading to a memory leak if the
/// resource contains a heap allocation. This includes `thread_local!`, `lazy_static`, `once_cell`,
/// etc. It's not the crate that matters, it's the heap resource.
///
/// Any `SafeLazy`s will be initialized the first time you use it, and will be dropped after your
/// plugin is dropped.
///
/// A `SafeLazy` must always be in a static variable. To do otherwise is undefined.
///
/// # Important
///
/// Any thread which accesses a safe static, mutex or no, must be killed inside your plugin's `Drop`
/// implementation. To allow otherwise is undefined.
pub struct SafeLazy<T>
where
    T: 'static,
{
    #[doc(hidden)]
    pub instance: SafeLazyInstance<T>,
    #[doc(hidden)]
    pub init_fn: fn() -> T,
}

unsafe impl<T> Sync for SafeLazy<T> where T: Send + Sync {}

#[doc(hidden)]
pub struct SafeLazyInstance<T> {
    instance: UnsafeCell<Option<T>>,
    once: Once,
}

impl<T> SafeLazyInstance<T> {
    #[doc(hidden)]
    pub const unsafe fn new() -> Self {
        Self {
            instance: UnsafeCell::new(None),
            once: Once::new(),
        }
    }
}

impl<T> Deref for SafeLazy<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let ptr = &*self as *const Self;
        self.instance.once.call_once(move || unsafe {
            *self.instance.instance.get() = Some((self.init_fn)());
            ALLOCATED
                .write()
                .as_mut()
                .unwrap()
                .push(Deallocator(Box::new(move || {
                    *(*ptr).instance.instance.get() = None
                })));
        });
        unsafe { (*self.instance.instance.get()).as_ref().unwrap() }
    }
}

/// An initially uninitialized resource that is safe to use in a HexChat plugin.
///
/// It is recommended that you not use any other kind of static resource crate, because any static
/// resource will not get dropped when your plugin is unloaded, leading to a memory leak if the
/// resource contains a heap allocation. This includes `thread_local!`, `lazy_static`, `once_cell`,
/// etc. It's not the crate that matters, it's the heap resource.
///
/// Any `SafeUninit`s will be initialized when the `init` function is called, and will be dropped
/// after your plugin is dropped.
///
/// A `SafeUninit` must always be in a static variable. To do otherwise is undefined.
///
/// # Important
///
/// Any thread which accesses a safe static, mutex or no, must be killed inside your plugin's `Drop`
/// implementation. To allow otherwise is undefined.
pub struct SafeUninit<T>
where
    T: 'static,
{
    instance: UnsafeCell<Option<T>>,
    once: Once,
}

impl<T> SafeUninit<T> {
    #[doc(hidden)]
    pub const unsafe fn new() -> Self {
        Self {
            instance: UnsafeCell::new(None),
            once: Once::new(),
        }
    }
    /// Initializes this `SafeUninit` with a value. Only works the first time it's called.
    pub fn init(&self, instance: T) {
        self.once
            .call_once(|| unsafe { *self.instance.get() = Some(instance) });
        let ptr = &*self as *const Self;
        ALLOCATED
            .write()
            .as_mut()
            .unwrap()
            .push(Deallocator(Box::new(move || unsafe {
                (*(*ptr).instance.get()) = None
            })));
    }
}

impl<T> Deref for SafeUninit<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let state = self.once.state();
        if state != OnceState::Done {
            panic!("Uninitialized `SafeUninit`");
        }
        unsafe { &*self.instance.get() }.as_ref().unwrap()
    }
}

pub(crate) static EXITING: AtomicBool = AtomicBool::new(false);

unsafe impl<T> Sync for SafeUninit<T> where T: Send + Sync {}

pub(crate) static ALLOCATED: RwLock<Option<Vec<Deallocator>>> = RwLock::new(None);
pub(crate) struct Deallocator(pub(crate) Box<dyn FnOnce()>);
unsafe impl Send for Deallocator {}
unsafe impl Sync for Deallocator {}
