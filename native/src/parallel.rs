use std::time::Instant;

use napi::{Error, Result};
use napi_derive::napi;
use tokio::task::{JoinSet, yield_now};

use crate::cpu::primes_in;
use crate::{ms_since, thread_name};

/// How many numbers a task checks before it offers its worker to other tasks.
const BATCH: u32 = 1_000_000;

/// A pause shorter than this is the scheduler's own overhead: far too brief
/// for another task to have had a batch-long turn on the thread.
const UNBROKEN_MS: f64 = 20.0;

/// One unbroken stretch of a task running on a thread.
#[napi(object)]
pub struct Stint {
  pub task: u32,
  pub thread: String,
  pub started_ms: f64,
  pub finished_ms: f64,
}

#[napi(object)]
pub struct ParallelReport {
  /// Wall-clock time for the whole batch.
  pub wall_ms: f64,
  /// Primes below the limit. Every task arrives at the same count.
  pub primes: u32,
  pub stints: Vec<Stint>,
}

/// Spawns `tasks` tasks that each count the primes below `limit` on their own,
/// and reports which thread ran each of them, and when.
///
/// These tasks compute rather than wait, so each one needs a thread. Idle
/// workers steal queued tasks from busy ones, and that is how a batch spawned
/// from a single worker ends up running on all of them.
#[napi]
pub async fn count_primes_in_parallel(
  tasks: u32,
  limit: u32,
  yield_between_batches: bool,
) -> Result<ParallelReport> {
  let origin = Instant::now();
  let mut running = JoinSet::new();

  for task in 0..tasks {
    running.spawn(count_in_batches(task, limit, yield_between_batches, origin));
  }

  let mut primes = 0;
  let mut stints = Vec::new();
  while let Some(joined) = running.join_next().await {
    let (counted, ran) = joined.map_err(|join_error| Error::from_reason(join_error.to_string()))?;
    primes = counted;
    stints.extend(ran);
  }

  Ok(ParallelReport {
    wall_ms: ms_since(origin),
    primes,
    stints,
  })
}

async fn count_in_batches(
  task: u32,
  limit: u32,
  yield_between_batches: bool,
  origin: Instant,
) -> (u32, Vec<Stint>) {
  let mut primes = 0;
  let mut stints: Vec<Stint> = Vec::new();

  for from in (0..limit).step_by(BATCH as usize) {
    let started_ms = ms_since(origin);
    primes += primes_in(from..limit.min(from.saturating_add(BATCH)));
    let finished_ms = ms_since(origin);
    let thread = thread_name();

    // Carrying straight on, on the same thread, extends the current stint.
    // Having waited behind another task or moved to another thread starts a new one.
    match stints.last_mut() {
      Some(last) if last.thread == thread && started_ms - last.finished_ms < UNBROKEN_MS => {
        last.finished_ms = finished_ms;
      }
      _ => stints.push(Stint {
        task,
        thread,
        started_ms,
        finished_ms,
      }),
    }

    // tokio can only switch tasks at an `.await`. Without this one the task
    // would hold its worker from start to finish, however many others queue up.
    if yield_between_batches {
      yield_now().await;
    }
  }

  (primes, stints)
}
