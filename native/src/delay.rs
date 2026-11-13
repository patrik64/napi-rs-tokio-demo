use std::time::{Duration, Instant};

use napi::{Error, Result, Status};
use napi_derive::napi;
use tokio::time;

use crate::ms_since;

/// `#[napi]` on an `async fn` exports a function that returns a `Promise`.
///
/// The future is polled by tokio, so the JS thread is free while it waits.
/// `Ok` resolves the promise; `Err` rejects it with a regular JS `Error`.
#[napi]
pub async fn delay(work_ms: u32, deadline_ms: Option<u32>) -> Result<f64> {
  let started = Instant::now();
  let work = time::sleep(Duration::from_millis(work_ms.into()));

  match deadline_ms {
    None => work.await,
    Some(deadline_ms) => time::timeout(Duration::from_millis(deadline_ms.into()), work)
      .await
      .map_err(|_| {
        Error::new(
          Status::Cancelled,
          format!("deadline of {deadline_ms} ms elapsed before the work finished"),
        )
      })?,
  }

  Ok(ms_since(started))
}
