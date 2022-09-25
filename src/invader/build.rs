extern crate embed_resource;

fn var(what : &str) -> String {
    std::env::var(what).unwrap()
}

use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
    let version = var("CARGO_PKG_VERSION");
    let version_dot = version.replace(".",",");
    let description = var("CARGO_PKG_DESCRIPTION");
    let pkg = var("CARGO_PKG_NAME");
    let count;

    // Get the commit hash
    let git_info = match std::process::Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output() {
        Ok(output) => {
            let hash = String::from_utf8(output.stdout).unwrap().trim().to_owned();
            count = String::from_utf8(std::process::Command::new("git").args(&["rev-list", "--count", "HEAD"]).output().unwrap().stdout).unwrap().trim().to_owned();
            format!("r{count}.{hash}")
        },
        Err(_) => {
            count = "0".to_owned();
            "unknown".to_owned()
        }
    };

    // Store version
    println!("cargo:rustc-env=invader_version={pkg} {version}.{git_info}");

    // 0x409 corresponds to English (U.S.)
    //
    // See https://docs.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource
    let language = 0x409;

    // Compile in windows.rc if Windows
    if var("CARGO_CFG_TARGET_OS") == "windows" {
        let windows_rc_path_raw = format!("{}/windows.rc", var("OUT_DIR"));
        let windows_rc_path = Path::new(&windows_rc_path_raw);

        write!(File::create(windows_rc_path).unwrap(),
"1 VERSIONINFO
FILEVERSION      {version_dot},{count}
PRODUCTVERSION   {version_dot},{count}
BEGIN
    BLOCK \"StringFileInfo\"
    BEGIN
        BLOCK \"040904B0\"
        BEGIN
            VALUE \"Comments\",         \"{description}\"
            VALUE \"CompanyName\",      \"Snowy Mouse\"
            VALUE \"FileDescription\",  \"{description}\"
            VALUE \"FileVersion\",      \"{version}.{count}\"
            VALUE \"InternalName\",     \"{pkg}\"
            VALUE \"OriginalFilename\", \"{pkg}.exe\"
            VALUE \"ProductName\",      \"{pkg}\"
            VALUE \"ProductVersion\",   \"{version}.{count}\"
            VALUE \"LegalCopyright\",   \"'22 Snowy Mouse\"
        END
    END
    BLOCK \"VarFileInfo\"
    BEGIN
        VALUE \"Translation\", 0x{language:X}, 1200
    END
END

IDI_ICON1 ICON DISCARDABLE \"icon/{pkg}.ico\"
").unwrap(); // end with a newline

        embed_resource::compile(windows_rc_path);
    }

    // We only need to change if Cargo.toml was modified, since that's where the version is stored
    println!("cargo:rerun-if-changed=Cargo.toml");
}
