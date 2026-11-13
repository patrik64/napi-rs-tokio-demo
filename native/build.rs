fn main() {
  napi_build::setup();

  // Expose the resolved versions of the two crates this showcase is about,
  // so the UI can display them without hardcoding anything.
  let lockfile = std::fs::read_to_string("Cargo.lock").unwrap_or_default();
  for krate in ["napi", "tokio"] {
    let needle = format!("name = \"{krate}\"\nversion = \"");
    let version = lockfile
      .split(&needle)
      .nth(1)
      .and_then(|rest| rest.split('"').next())
      .unwrap_or("unknown");
    println!("cargo:rustc-env={}_VERSION={version}", krate.to_uppercase());
  }
  println!("cargo:rerun-if-changed=Cargo.lock");
}
