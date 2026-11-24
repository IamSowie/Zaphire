//! Run closures on CEF threads.

use cef::*;
use std::sync::{Arc, Mutex};

type Closure = Arc<Mutex<Option<Box<dyn FnOnce() + Send>>>>;

wrap_task! {
    struct OnceTask {
        f: Closure,
    }

    impl Task {
        fn execute(&self) {
            let f = self.f.lock().ok().and_then(|mut f| f.take());
            if let Some(f) = f {
                f();
            }
        }
    }
}

/// Run `f` on the browser UI thread after `delay_ms` (0 = as soon as possible).
pub fn on_ui(delay_ms: i64, f: impl FnOnce() + Send + 'static) {
    let mut task = OnceTask::new(Arc::new(Mutex::new(Some(Box::new(f)))));
    if delay_ms > 0 {
        post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
    } else {
        post_task(ThreadId::UI, Some(&mut task));
    }
}

/// Seconds since the browser started; the clock used for tab suspension.
pub fn now_secs() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs()
}
