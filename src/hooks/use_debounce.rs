use std::time::{Duration, Instant};

use freya::prelude::*;

pub fn use_debounce<T, F>(time: Duration, cb: F) -> impl FnMut(T)
where
    F: Fn(T),
{
    let mut last_invoke = use_signal::<Option<Instant>>(|| None);

    move |e: T| {
        if let Some(got_last_invoke) = last_invoke() {
            if got_last_invoke.elapsed() > time {
                last_invoke.set(Some(Instant::now()));
                cb(e);
            }
        } else {
            last_invoke.set(Some(Instant::now()));
            cb(e);
        }
    }
}
