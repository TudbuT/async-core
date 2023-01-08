use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A future which returns Pending once, and returns Ready the next time it is polled. Used to
/// interrupt a running task to make space for others to run.
pub struct YieldFuture(bool);

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.get_mut().0 = true;
            Poll::Pending
        }
    }
}

/// Interrupts the current task and yields for the ohers. This uses a YieldFuture
pub fn yield_now() -> YieldFuture {
    YieldFuture(false)
}
