#![deny(clippy::all)]

//! Native half of the showcase. Each module backs one demo on the page, and
//! every `#[napi]` item in them becomes an export of the generated `index.js`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use napi::bindgen_prelude::create_custom_tokio_runtime;
use napi_derive::module_init;
use tokio::runtime::Builder;

mod cpu;
mod delay;
mod fan_out;
mod parallel;
mod probe;
mod runtime;
mod stream;

/// Runs once when Node loads the addon, before any export can be called.
///
/// With the `tokio_rt` feature napi-rs would lazily create a default
/// multi-threaded runtime. Handing it our own lets us configure it — here we
/// number the threads so the demos can report where each task actually ran.
#[module_init]
fn init() {
  let runtime = Builder::new_multi_thread()
    .enable_all()
    .thread_name_fn(|| {
      static NEXT: AtomicUsize = AtomicUsize::new(0);
      format!("tokio-{}", NEXT.fetch_add(1, Ordering::Relaxed))
    })
    .build()
    .expect("failed to build the tokio runtime");

  create_custom_tokio_runtime(runtime);
}

/// Name of the OS thread the caller is running on.
///
/// `init` names every tokio thread, so an unnamed one can only be the JS
/// thread calling a synchronous export.
fn thread_name() -> String {
  std::thread::current()
    .name()
    .unwrap_or("JS thread")
    .to_owned()
}

/// Milliseconds elapsed since `origin`, as the `f64` JavaScript expects.
fn ms_since(origin: Instant) -> f64 {
  origin.elapsed().as_secs_f64() * 1000.0
}
