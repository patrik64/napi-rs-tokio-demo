use std::f64::consts::TAU;
use std::time::{Duration, Instant};

use napi::bindgen_prelude::spawn;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Error, Result, Status};
use napi_derive::napi;
use tokio::sync::mpsc;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::ms_since;

#[napi(object)]
pub struct StreamOptions {
  pub producers: u32,
  pub samples_per_producer: u32,
  pub interval_ms: u32,
}

/// Becomes a discriminated union in TypeScript, tagged by `type`.
#[napi(discriminant_case = "lowercase")]
pub enum StreamEvent {
  Sample {
    producer: u32,
    seq: u32,
    value: f64,
    at_ms: f64,
  },
  End {
    delivered: u32,
    cancelled: bool,
  },
}

#[napi]
pub struct StreamHandle {
  cancel: CancellationToken,
}

#[napi]
impl StreamHandle {
  /// Stops every task belonging to this stream. Safe to call more than once.
  #[napi]
  pub fn stop(&self) {
    self.cancel.cancel();
  }
}

/// Starts background tasks that push events to `on_event` until they run out
/// of samples or the returned handle is stopped.
///
/// Several producers feed one bounded channel; a single forwarder drains it
/// and calls back into JavaScript.
///
/// A `ThreadsafeFunction` is a JS callback that may be invoked from any
/// thread. Its two const generics say: pass the event alone (no leading `err`
/// argument), and don't let a pending callback keep the Node process alive.
#[napi]
pub fn start_stream(
  options: StreamOptions,
  on_event: ThreadsafeFunction<StreamEvent, (), StreamEvent, Status, false, true>,
) -> Result<StreamHandle> {
  if options.producers == 0 || options.interval_ms == 0 {
    return Err(Error::new(
      Status::InvalidArg,
      "expected producers >= 1 and intervalMs >= 1",
    ));
  }

  let cancel = CancellationToken::new();
  let origin = Instant::now();
  let (events, mut inbox) = mpsc::channel(64);

  // This function is synchronous and runs on the JS thread, outside the
  // runtime, so tasks are started with napi's `spawn` rather than `tokio::spawn`.
  for producer in 0..options.producers {
    let produce = produce(
      producer,
      options.samples_per_producer,
      options.interval_ms,
      origin,
      events.clone(),
    );
    let cancel = cancel.clone();
    spawn(async move {
      cancel.run_until_cancelled(produce).await;
    });
  }
  // Only the producers hold senders now, so the channel closes when they finish.
  drop(events);

  let forwarder_cancel = cancel.clone();
  spawn(async move {
    let mut delivered = 0;
    while let Some(Some(event)) = forwarder_cancel.run_until_cancelled(inbox.recv()).await {
      on_event.call(event, ThreadsafeFunctionCallMode::NonBlocking);
      delivered += 1;
    }

    let end = StreamEvent::End {
      delivered,
      cancelled: forwarder_cancel.is_cancelled(),
    };
    on_event.call(end, ThreadsafeFunctionCallMode::NonBlocking);
  });

  Ok(StreamHandle { cancel })
}

/// Emits a noisy sine wave; each producer ticks at its own pace.
async fn produce(
  producer: u32,
  samples: u32,
  interval_ms: u32,
  origin: Instant,
  events: mpsc::Sender<StreamEvent>,
) {
  let period = Duration::from_millis(u64::from(interval_ms) * u64::from(producer + 1));
  let mut ticker = time::interval(period);

  for seq in 0..samples {
    ticker.tick().await;

    let at_ms = ms_since(origin);
    let wave = (at_ms / 1000.0 * TAU / f64::from(producer + 2)).sin();
    let sample = StreamEvent::Sample {
      producer,
      seq,
      value: wave + (fastrand::f64() - 0.5) * 0.25,
      at_ms,
    };

    // `send` waits while the channel is full and fails once the receiver is gone.
    if events.send(sample).await.is_err() {
      return;
    }
  }
}
