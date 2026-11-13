use std::sync::Arc;
use std::time::{Duration, Instant};

use napi::{Error, Result, Status};
use napi_derive::napi;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time;

use crate::{ms_since, thread_name};

#[napi(object)]
pub struct TaskSpan {
  pub id: u32,
  /// When the task got its permit and started working.
  pub started_ms: f64,
  pub finished_ms: f64,
  /// The worker thread that completed the task.
  pub thread: String,
}

#[napi(object)]
pub struct FanOutReport {
  /// Wall-clock time for the whole batch.
  pub wall_ms: f64,
  /// Sum of every task's own duration — what running them one by one would cost.
  pub work_ms: f64,
  /// One entry per task, ordered by start time.
  pub spans: Vec<TaskSpan>,
}

/// Spawns `tasks` tokio tasks that each wait a random time, with at most
/// `max_concurrency` of them working at once, and reports when each one ran.
#[napi]
pub async fn fan_out(
  tasks: u32,
  max_concurrency: u32,
  min_ms: u32,
  max_ms: u32,
) -> Result<FanOutReport> {
  if max_concurrency == 0 || min_ms > max_ms {
    return Err(Error::new(
      Status::InvalidArg,
      "expected maxConcurrency >= 1 and minMs <= maxMs",
    ));
  }

  let origin = Instant::now();
  let permits = Arc::new(Semaphore::new(max_concurrency as usize));
  let mut running = JoinSet::new();

  for id in 0..tasks {
    let permits = permits.clone();
    let work = Duration::from_millis(fastrand::u32(min_ms..=max_ms).into());

    running.spawn(async move {
      let _permit = permits.acquire().await.expect("semaphore is never closed");
      let started_ms = ms_since(origin);
      time::sleep(work).await;

      TaskSpan {
        id,
        started_ms,
        finished_ms: ms_since(origin),
        thread: thread_name(),
      }
    });
  }

  let mut spans = Vec::with_capacity(tasks as usize);
  while let Some(joined) = running.join_next().await {
    spans.push(joined.map_err(|join_error| Error::from_reason(join_error.to_string()))?);
  }
  spans.sort_by(|a, b| a.started_ms.total_cmp(&b.started_ms));

  Ok(FanOutReport {
    wall_ms: ms_since(origin),
    work_ms: spans
      .iter()
      .map(|span| span.finished_ms - span.started_ms)
      .sum(),
    spans,
  })
}
