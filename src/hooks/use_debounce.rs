use std::time::{Duration, Instant};

use freya::prelude::*;

pub fn use_debounce<T, F>(time: Duration, cb: F) -> impl FnMut(T)
where
    F: Fn(T),
{
    let mut last_invoke = use_signal(|| Instant::now());

    move |e: T| {
        if last_invoke().elapsed() > time {
            last_invoke.set(Instant::now());
            cb(e);
        }
    }
}
