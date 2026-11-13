use napi::bindgen_prelude::within_runtime_if_available;
use napi_derive::napi;
use tokio::runtime::Handle;

#[napi(object)]
pub struct RuntimeInfo {
  pub napi_version: String,
  pub tokio_version: String,
  pub workers: u32,
}

#[napi(object)]
pub struct RuntimeMetrics {
  /// Tasks that were spawned and have not finished yet.
  pub alive_tasks: u32,
  /// Tasks waiting in the queue shared by all workers.
  pub global_queue_depth: u32,
  /// Total time each worker has spent executing tasks, in milliseconds.
  /// tokio adds to it whenever the worker pauses, not while a task is running.
  pub worker_busy_ms: Vec<f64>,
  /// Whether each worker is running something at this very moment.
  pub worker_active: Vec<bool>,
}

/// Static facts about the addon. The versions are baked in by `build.rs`.
#[napi]
pub fn runtime_info() -> RuntimeInfo {
  RuntimeInfo {
    napi_version: env!("NAPI_VERSION").to_owned(),
    tokio_version: env!("TOKIO_VERSION").to_owned(),
    workers: with_handle(|handle| handle.metrics().num_workers() as u32),
  }
}

/// A snapshot of the runtime, cheap enough to poll several times a second.
#[napi]
pub fn runtime_metrics() -> RuntimeMetrics {
  with_handle(|handle| {
    let metrics = handle.metrics();
    let workers = 0..metrics.num_workers();

    RuntimeMetrics {
      alive_tasks: metrics.num_alive_tasks() as u32,
      global_queue_depth: metrics.global_queue_depth() as u32,
      worker_busy_ms: workers
        .clone()
        .map(|worker| metrics.worker_total_busy_duration(worker).as_secs_f64() * 1000.0)
        .collect(),
      // The counter goes up by one on every park and every unpark, so an even
      // value means the worker is currently awake.
      worker_active: workers
        .map(|worker| metrics.worker_park_unpark_count(worker) % 2 == 0)
        .collect(),
    }
  })
}

/// Synchronous exports run on the JS thread, which is not a tokio thread, so
/// `Handle::current()` would panic there. napi-rs can enter the runtime for us.
fn with_handle<T>(read: impl FnOnce(&Handle) -> T) -> T {
  within_runtime_if_available(|| read(&Handle::current()))
}
