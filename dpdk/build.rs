use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    if env::var_os("CARGO_FEATURE_REAL_DPDK").is_none() {
        return;
    }

    let output = Command::new("pkg-config")
        .args(["--libs", "--cflags", "libdpdk"])
        .output()
        .expect("failed to run pkg-config for libdpdk");

    if !output.status.success() {
        panic!(
            "pkg-config could not find libdpdk. Install DPDK development files or set PKG_CONFIG_PATH.\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let flags = String::from_utf8(output.stdout).expect("pkg-config output was not UTF-8");
    for flag in flags.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else if let Some(path) = flag.strip_prefix("-Wl,-rpath,") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{path}");
        }
    }
}
