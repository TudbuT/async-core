fn main() {
    use microasync::sync;
    use microasync_rt::QueuedRuntime;

    sync(QueuedRuntime::new_with(other_crate::wait_and_print(
        500, "hi",
    )));
}

mod other_crate {
    use async_core::{get_current_runtime, Runtime};

    pub async fn wait_and_print(ms: u64, s: &'static str) {
        print("waiting...").await;
        get_current_runtime().await.sleep_ms(ms).await;
        print("done waiting.").await;
        let mut rt = get_current_runtime().await;
        let f = rt.spawn(print(s));
        println!("spawned print task");
        f.await;
        println!("print task done");
    }

    pub async fn print(s: &'static str) {
        println!("{}", s);
    }
}
