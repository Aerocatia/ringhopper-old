// Get the version.
//
// NOTE: We have no means to detect git being changed yet, so this will rerun if any file has changed!

fn var(what : &str) -> String {
    std::env::var(what).unwrap()
}

fn main() {
    let version = var("CARGO_PKG_VERSION");
    let pkg = var("CARGO_PKG_NAME");

    // Get the commit hash
    let git_info = match std::process::Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output() {
        Ok(output) => {
            let hash = String::from_utf8(output.stdout).unwrap().trim().to_owned();
            let count = String::from_utf8(std::process::Command::new("git").args(&["rev-list", "--count", "HEAD"]).output().unwrap().stdout).unwrap().trim().to_owned();
            format!("r{count}.{hash}")
        },
        Err(_) => {
            "unknown".to_owned()
        }
    };

    // Store version
    println!("cargo:rustc-env=ringhopper_version={pkg} {version}.{git_info}");
}
