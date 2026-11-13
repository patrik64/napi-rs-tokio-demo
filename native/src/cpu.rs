use std::ops::Range;
use std::time::Instant;

use napi::{Error, Result};
use napi_derive::napi;
use tokio::task;

use crate::{ms_since, thread_name};

#[napi(object)]
pub struct PrimeCount {
  pub primes: u32,
  pub elapsed_ms: f64,
  /// The OS thread that did the counting.
  pub thread: String,
}

/// Deliberately naive, CPU-bound work: trial division, no awaits, no yielding.
pub(crate) fn primes_in(numbers: Range<u32>) -> u32 {
  numbers
    .filter(|&n| n >= 2 && (2..).take_while(|&d| d <= n / d).all(|d| n % d != 0))
    .count() as u32
}

fn count_primes(limit: u32) -> PrimeCount {
  let started = Instant::now();
  let primes = primes_in(0..limit);

  PrimeCount {
    primes,
    elapsed_ms: ms_since(started),
    thread: thread_name(),
  }
}

/// 1. A synchronous export runs on the JS thread. Nothing else in Node — no
///    timers, no other requests — makes progress until it returns.
#[napi]
pub fn count_primes_sync(limit: u32) -> PrimeCount {
  count_primes(limit)
}

/// 2. An `async fn` runs on a tokio worker, so Node stays responsive. But the
///    work never yields, so that worker can't run any other task meanwhile.
#[napi]
pub async fn count_primes_on_worker(limit: u32) -> PrimeCount {
  count_primes(limit)
}

/// 3. `spawn_blocking` moves the work to tokio's dedicated blocking pool,
///    keeping both Node's event loop and the async workers free.
#[napi]
pub async fn count_primes_blocking(limit: u32) -> Result<PrimeCount> {
  task::spawn_blocking(move || count_primes(limit))
    .await
    .map_err(|join_error| Error::from_reason(join_error.to_string()))
}
