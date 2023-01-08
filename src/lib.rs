//! This crate provides wrappers for async operations in order to help standardize rust async
//! runtimes.
//!
//! SAFETY: The current runtime *must* be cleaned up after each tick of a runtime, and must
//! therefore live shorter than the runtime itself as the runtime can not be dropped while it
//! is running. Without this, async-core causes undefined behavior.
//!
//! SAFETY: In a no_std environment, creating multiple threads is inherently unsafe as there is no
//! safe API wrapping threads. I assume you know what you are doing if you are using this with
//! multiple threads in a no_std environment. Please do not share runtimes between threads this way.
//!
//! SAFETY: DO NOT USE process::exit!
//!
//! ## How to use a runtime:
//!
//! - Import the re-exported Runtime and get_current_runtime.
//! - (Skip this if you are a library developer and want your users to control the runtime) Create
//!   a runtime like you normally would
//! - Don't use that runtime's specifics if you can somehow avoid it. Use get_current_runtime and
//!   the functions provided in the returned RuntimeWrapper
//!
//! ## How to implement a runtime:
//!
//! - Re-export everything so your users can use async_core without needing to add it as a
//!   dependency.
//! - In the runtime, call set_current_runtime and clear_current_runtime:
//! ```
//! set_current_runtime(self);
//! let result = execute_futures(); // Do your cool runtime things here
//! clear_current_runtime();
//! result
//! ```
//! - If your crate has a no_std feature, link that with this crate's no_std feature.
//! - Make absolutely sure you call set_current_runtime and clear_current_runtime.

#![no_std]
#[cfg(not(feature = "no_std"))]
extern crate std;

extern crate alloc;

mod runtime;
pub use runtime::*;

mod defer;
#[cfg(not(feature = "no_std"))]
pub use defer::*;

mod yielding;
pub use yielding::*;

mod joiner;
pub use joiner::*;

use alloc::boxed::Box;
use core::pin::Pin;
use core::{cell::RefCell, future::Future};
use core::{mem, mem::ManuallyDrop, panic};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
#[inline]
pub fn prep<'a, T>(future: impl Future<Output = T> + 'a) -> BoxFuture<'a, T> {
    Box::pin(future)
}

#[cfg(feature = "no_std")]
mod no_std_util {
    pub struct ForceShare<T>(pub T);
    unsafe impl<T> Send for ForceShare<T> {}
    unsafe impl<T> Sync for ForceShare<T> {}
    impl<T> ForceShare<T> {
        pub fn with<F, R>(&self, fun: F) -> R
        where
            F: FnOnce(&T) -> R,
        {
            fun(&self.0)
        }
    }
}

#[cfg(feature = "no_std")]
static CURRENT_RUNTIME: no_std_util::ForceShare<
    RefCell<Option<ManuallyDrop<Box<dyn InternalRuntime>>>>,
> = no_std_util::ForceShare(RefCell::new(None));

#[cfg(not(feature = "no_std"))]
std::thread_local! {
    static CURRENT_RUNTIME: RefCell<Option<ManuallyDrop<Box<dyn InternalRuntime>>>> = RefCell::new(None);
}

/// This gets the currently running runtime. PANICS IF IT IS CALLED FROM OUTSIDE THE RUNTIME.
pub async fn get_current_runtime<'a>() -> RuntimeWrapper<'a> {
    CURRENT_RUNTIME.with(|x| {
        if let Some(x) = x.borrow_mut().as_mut() {
            unsafe { RuntimeWrapper(&mut *(x.as_mut() as *mut _)) }
        } else {
            panic!(
                "get_current_runtime MUST only be called from a future running within a Runtime!"
            )
        }
    })
}

/// This sets the currently running runtime. MUST *ONLY* BE CALLED BY A RUNTIME WHEN IT IS ABOUT TO
/// START EXECUTING, AND MUST BE CLEARED AFTERWARDS.
pub fn set_current_runtime(runtime: &mut dyn InternalRuntime) {
    CURRENT_RUNTIME.with(move |x| {
        *x.borrow_mut() = Some(unsafe {
            ManuallyDrop::new(Box::from_raw(
                mem::transmute::<_, *mut dyn InternalRuntime>(runtime),
            ))
        })
    })
}

/// This clears the currently running runtime. MUST *ONLY* BE CALLED BY A RUNTIME WHEN IT WILL
/// (temporarily of permanently) STOP EXECUTING FUTURES. MUST FOLLOW A set_current_runtime CALL.
/// IF THE RUNTIME STARTS TO BE USED AGAIN, set_current_runtime MUST BE CALLED AGAIN.
pub fn clear_current_runtime() {
    CURRENT_RUNTIME.with(|x| *x.borrow_mut() = None)
}
