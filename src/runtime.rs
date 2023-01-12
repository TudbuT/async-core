extern crate alloc;

use crate::BoxFuture;
use alloc::boxed::Box;
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

/// Stable wrapper of `!` type for the Stop struct.
pub enum Never {}
impl Never { pub fn into(self) -> ! { loop {} } }

/// A never-completing future, used in stopping the runtime. Returns stable equivalent of `!`
pub struct Stop;

impl Future for Stop {
    type Output = Never;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending // Return access to the runtime immediately
    }
}

/// Future equivalent of a join handle
pub struct SpawnedFuture<'a>(u64, &'a mut dyn InternalRuntime);

impl<'a> Future for SpawnedFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        if me.1.contains(me.0) {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Trait to be implemented by async runtimes
pub trait InternalRuntime {
    /// Adds a new future to the queue to be completed.
    fn push_boxed(&mut self, future: BoxFuture<'static, ()>) -> u64;
    /// Returns if a future by some ID is still running.
    fn contains(&mut self, id: u64) -> bool;
    /// Asynchronously sleeps
    fn sleep<'a>(&self, duration: Duration) -> BoxFuture<'a, ()>;
    /// Stops the runtime
    fn stop(&mut self) -> Stop;
}
/// Auto-trait with the methods that will actually be called by the users of the runtime.
pub trait Runtime<'a> {
    /// Adds a new future to the queue to be completed.
    fn push(&mut self, future: impl Future<Output = ()> + 'static) {
        Runtime::push_boxed(self, Box::pin(future))
    }
    /// Adds a new future to the queue to be completed.
    fn push_boxed(&mut self, future: BoxFuture<'static, ()>);

    /// Adds a new future to the queue to be completed and returns a future waiting for the added
    /// future's completion.
    fn spawn(&'a mut self, future: impl Future<Output = ()> + 'static) -> SpawnedFuture<'a> {
        Runtime::spawn_boxed(self, Box::pin(future))
    }
    /// Adds a new future to the queue to be completed and returns a future waiting for the added
    /// future's completion.
    fn spawn_boxed(&'a mut self, future: BoxFuture<'static, ()>) -> SpawnedFuture<'a>;

    /// Asynchronously sleeps
    fn sleep<'b>(&'a self, duration: Duration) -> BoxFuture<'b, ()>;
    /// Asynchronously sleeps some amount of milliseconds
    fn sleep_ms<'b>(&'a self, amount: u64) -> BoxFuture<'b, ()> {
        self.sleep(Duration::from_millis(amount))
    }

    /// Stops the runtime. This does not exit the process.
    fn stop(&mut self) -> Stop;
}

/// Wrapper for anything that implements InternalRuntime, used to add a Runtime impl.
pub struct RuntimeWrapper<'a>(pub(crate) &'a mut dyn InternalRuntime);

/// Owned wrapper for anything that implements InternalRuntime, used to add a Runtime impl.
pub struct OwnedRuntime<'a>(RuntimeWrapper<'a>, pub(crate) Box<dyn InternalRuntime + 'a>);

impl<'a> Deref for OwnedRuntime<'a> {
    type Target = RuntimeWrapper<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Runtime<'a> for RuntimeWrapper<'a> {
    fn push_boxed(&mut self, future: BoxFuture<'static, ()>) {
        InternalRuntime::push_boxed(self.0, future);
    }

    fn spawn_boxed(&'a mut self, future: BoxFuture<'static, ()>) -> SpawnedFuture<'a> {
        SpawnedFuture(InternalRuntime::push_boxed(self.0, future), self.0)
    }

    fn sleep<'b>(&'a self, duration: Duration) -> BoxFuture<'b, ()> {
        InternalRuntime::sleep(self.0, duration)
    }

    fn stop(&mut self) -> Stop {
        InternalRuntime::stop(self.0)
    }
}

impl<'a, T: InternalRuntime + Sized> Runtime<'a> for T {
    fn push_boxed(&mut self, future: BoxFuture<'static, ()>) {
        InternalRuntime::push_boxed(self, future);
    }

    fn spawn_boxed(&'a mut self, future: BoxFuture<'static, ()>) -> SpawnedFuture<'a> {
        SpawnedFuture(InternalRuntime::push_boxed(self, future), self)
    }

    fn sleep<'b>(&'a self, duration: Duration) -> BoxFuture<'b, ()> {
        InternalRuntime::sleep(self, duration)
    }

    fn stop(&mut self) -> Stop {
        InternalRuntime::stop(self)
    }
}

/// Trait to construct a runtime
pub trait StartableRuntime<'a>: InternalRuntime + Sized + 'a {
    /// Constructs some new runtime
    fn new() -> OwnedRuntime<'a> {
        let mut bx = Box::new(Self::construct());
        OwnedRuntime(
            RuntimeWrapper(unsafe { (bx.as_mut() as *mut dyn InternalRuntime).as_mut().unwrap() }),
            bx,
        )
    }

    /// Internal function to make a new runtime. Only to be used by new() to create an
    /// OwnedRuntime. Automatically implemented for T where T: Default
    fn construct() -> Self;
}

impl<'a, T: InternalRuntime + Sized + 'a> StartableRuntime<'a> for T
where
    T: Default,
{
    fn construct() -> Self {
        Self::default()
    }
}
