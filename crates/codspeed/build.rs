fn main() {
    println!("cargo:rustc-check-cfg=cfg(use_instrument_hooks)");

    println!("cargo:rerun-if-changed=instrument-hooks/dist/core.c");
    println!("cargo:rerun-if-changed=instrument-hooks/includes/core.h");
    println!("cargo:rerun-if-changed=build.rs");

    collect_rustc_info();
    collect_cargo_info();
    collect_build_profile_info();

    let mut build = cc::Build::new();
    build
        .file("instrument-hooks/dist/core.c")
        .include("instrument-hooks/includes")
        .std("c11")
        // We generated the C code from Zig, which contains some warnings
        // that can be safely ignored.
        .flag("-Wno-format")
        .flag("-Wno-format-security")
        .flag("-Wno-unused-but-set-variable")
        .flag("-Wno-unused-const-variable")
        .flag("-Wno-type-limits")
        .flag("-Wno-uninitialized")
        .flag("-Wno-overflow")
        .flag("-Wno-unused-function")
        .flag("-Wno-constant-conversion")
        .flag("-Wno-incompatible-pointer-types")
        .flag("-Wno-unterminated-string-initialization")
        .opt_level(3);

    let result = build.try_compile("instrument_hooks");
    match result {
        Ok(_) => println!("cargo:rustc-cfg=use_instrument_hooks"),
        Err(e) => {
            let compiler = build.try_get_compiler().expect("Failed to get C compiler");

            eprintln!("\n\nWARNING: Failed to compile instrument-hooks native library with cc-rs.");
            eprintln!("The library will still compile, but instrument-hooks functionality will be disabled.");
            eprintln!("Compiler information: {compiler:?}");
            eprintln!("Compilation error: {e}\n");

            println!("cargo:warning=Failed to compile instrument-hooks native library with cc-rs. Continuing with noop implementation.");
        }
    }
}

/// Collect build profile and performance-relevant compiler options at build time.
/// These env var names must be kept in sync with `src/instrument_hooks/mod.rs`.
///
/// Reference: <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts>
fn collect_build_profile_info() {
    // PROFILE is set by Cargo to the name of the profile being built (e.g. "release", "bench", "debug").
    if let Ok(profile) = std::env::var("PROFILE") {
        println!("cargo:rustc-env=CODSPEED_BUILD_PROFILE={profile}");
    }

    // OPT_LEVEL is set by Cargo to the optimization level (0, 1, 2, 3, s, z).
    if let Ok(opt_level) = std::env::var("OPT_LEVEL") {
        println!("cargo:rustc-env=CODSPEED_BUILD_OPT_LEVEL={opt_level}");
    }

    // Extract performance-relevant codegen flags from rustflags.
    //
    // Cargo uses two mutually exclusive sources for rustflags (see
    // <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads>):
    //   - RUSTFLAGS env var (space-separated, takes priority)
    //   - CARGO_ENCODED_RUSTFLAGS env var (0x1f-separated, used when RUSTFLAGS is not set)
    // We check both in priority order to accurately represent the flags used to build.
    let rustflags: Vec<String> = if let Ok(raw) = std::env::var("RUSTFLAGS") {
        raw.split_whitespace().map(String::from).collect()
    } else if let Ok(encoded) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        encoded.split('\x1f').map(String::from).collect()
    } else {
        Vec::new()
    };

    if let Some(target_cpu) = extract_codegen_flag(&rustflags, "target-cpu") {
        println!("cargo:rustc-env=CODSPEED_BUILD_TARGET_CPU={target_cpu}");
    }
}

/// Collect rustc toolchain info at build time and expose as env vars.
/// These env var names must be kept in sync with `src/instrument_hooks/mod.rs`.
fn collect_rustc_info() {
    let Ok(output) = std::process::Command::new("rustc")
        .args(["--version", "--verbose"])
        .output()
    else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("rustc ") {
            println!("cargo:rustc-env=CODSPEED_RUSTC_VERSION={rest}");
        } else if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            let env_key = match key {
                "release" => Some("CODSPEED_RUSTC_RELEASE"),
                "LLVM version" => Some("CODSPEED_RUSTC_LLVM_VERSION"),
                _ => None,
            };
            if let Some(env_key) = env_key {
                println!("cargo:rustc-env={env_key}={value}");
            }
        }
    }
}

/// Collect cargo version at build time and expose as an env var.
/// This env var name must be kept in sync with `src/instrument_hooks/mod.rs`.
fn collect_cargo_info() {
    let Ok(output) = std::process::Command::new("cargo")
        .arg("--version")
        .output()
    else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(rest) = stdout.trim().strip_prefix("cargo ") {
        println!("cargo:rustc-env=CODSPEED_CARGO_VERSION={rest}");
    }
}

/// Extract a codegen option value from a list of rustc flags.
///
/// Handles all three forms that rustc accepts:
///   - `-Ckey=value` (no space)
///   - `-C` `key=value` (space between -C and the option, split into two entries)
///   - `-C` `key` `value` (space between key and value, split into three entries) — only for flags that accept it
///
/// Returns the value for the first match of `key`.
fn extract_codegen_flag(flags: &[String], key: &str) -> Option<String> {
    let mut iter = flags.iter();
    while let Some(flag) = iter.next() {
        // Form: -Ckey=value
        if let Some(rest) = flag.strip_prefix("-C") {
            if let Some(value) = rest.strip_prefix(key).and_then(|s| s.strip_prefix('=')) {
                return Some(value.to_string());
            }
            // rest is not our key, skip
            continue;
        }

        // Form: -C key=value  or  -C key value
        if flag == "-C" {
            if let Some(next) = iter.next() {
                if let Some(value) = next.strip_prefix(key).and_then(|s| s.strip_prefix('=')) {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}
