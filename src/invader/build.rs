fn var(what : &str) -> String {
    std::env::var(what).unwrap()
}

fn main() {
    let version = var("CARGO_PKG_VERSION");
    let pkg = var("CARGO_PKG_NAME");

    // Store version
    println!("cargo:rustc-env=invader_version={} {}", pkg, version);

    // We only need to change if Cargo.toml was modified, since that's where the version is stored
    println!("cargo:rerun-if-changed=Cargo.toml");
}
