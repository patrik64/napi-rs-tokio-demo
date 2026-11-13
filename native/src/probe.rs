use std::io;
use std::time::{Duration, Instant};

use napi::{Error, Result};
use napi_derive::napi;
use tokio::net::{self, TcpStream};
use tokio::time;

use crate::ms_since;

#[napi(object)]
pub struct ProbeResult {
  /// The `host:port` that was probed.
  pub target: String,
  /// The address the host resolved to, if DNS succeeded in time.
  pub address: Option<String>,
  pub resolve_ms: Option<f64>,
  pub connect_ms: Option<f64>,
  pub total_ms: f64,
  pub error: Option<String>,
}

/// Resolves and opens a TCP connection to every target at the same time,
/// giving each one `timeout_ms` to finish. Results keep the input order.
#[napi]
pub async fn probe(targets: Vec<String>, timeout_ms: u32) -> Result<Vec<ProbeResult>> {
  let timeout = Duration::from_millis(timeout_ms.into());

  // Spawning first and awaiting afterwards is what makes the probes overlap.
  let probes: Vec<_> = targets
    .into_iter()
    .map(|target| tokio::spawn(probe_one(target, timeout)))
    .collect();

  let mut results = Vec::with_capacity(probes.len());
  for probe in probes {
    results.push(
      probe
        .await
        .map_err(|join_error| Error::from_reason(join_error.to_string()))?,
    );
  }
  Ok(results)
}

async fn probe_one(target: String, timeout: Duration) -> ProbeResult {
  let started = Instant::now();
  let mut result = ProbeResult {
    target: target.clone(),
    address: None,
    resolve_ms: None,
    connect_ms: None,
    total_ms: 0.0,
    error: None,
  };

  let attempt = async {
    let address = net::lookup_host(&target)
      .await?
      .next()
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "host has no addresses"))?;
    let resolved_ms = ms_since(started);
    result.address = Some(address.to_string());
    result.resolve_ms = Some(resolved_ms);

    TcpStream::connect(address).await?;
    result.connect_ms = Some(ms_since(started) - resolved_ms);
    io::Result::Ok(())
  };

  result.error = match time::timeout(timeout, attempt).await {
    Ok(Ok(())) => None,
    Ok(Err(io_error)) => Some(io_error.to_string()),
    Err(_) => Some(format!("timed out after {} ms", timeout.as_millis())),
  };
  result.total_ms = ms_since(started);
  result
}
