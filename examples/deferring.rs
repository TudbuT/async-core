fn main() {
    use microasync::sync;
    use microasync_rt::QueuedRuntime;

    sync(QueuedRuntime::new_with(
        other_crate::wait_and_print_and_exit(1000, "hi"),
    ));
}

mod other_crate {
    use async_core::{defer, get_current_runtime, Runtime};
    use std::{process, thread, time::Duration};

    pub async fn is_it_still_running() {
        get_current_runtime().await.sleep_ms(250).await;
        print("still running").await;
        get_current_runtime().await.push(is_it_still_running());
    }

    pub async fn wait_and_print_and_exit(ms: u64, s: &'static str) {
        is_it_still_running().await;
        print("waiting...").await;
        // for whatever reason, let's wait in a blocking manner.
        defer(|ms| thread::sleep(Duration::from_millis(ms)), ms).await;
        print("done waiting.").await;
        let mut rt = get_current_runtime().await;
        let f = rt.spawn(print(s));
        println!("spawned print task");
        f.await;
        println!("print task done");
        process::exit(0);
    }

    pub async fn print(s: &'static str) {
        println!("{}", s);
    }
}
