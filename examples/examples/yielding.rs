use async_core::{get_current_runtime, yield_now, Runtime};
use microasync::sync;
use microasync_rt::QueuedRuntime;

fn main() {
    sync(QueuedRuntime::new_with(async_main()));
}

async fn recursion() {
    println!("recursion!");
    get_current_runtime().await.push(recursion());
}

async fn looping() {
    loop {
        println!("looping!");
        yield_now().await;
    }
}

async fn async_main() {
    get_current_runtime().await.push(recursion());
    get_current_runtime().await.push(looping());
}
