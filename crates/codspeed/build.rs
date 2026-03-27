fn main() {
    println!("cargo:rustc-check-cfg=cfg(use_instrument_hooks)");

    println!("cargo:rerun-if-changed=instrument-hooks/dist/core.c");
    println!("cargo:rerun-if-changed=instrument-hooks/includes/core.h");
    println!("cargo:rerun-if-changed=build.rs");

    collect_rustc_info();
    collect_cargo_info();

    let mut build = cc::Build::new();
    build
        .flag("-std=c11")
        .file("instrument-hooks/dist/core.c")
        .include("instrument-hooks/includes")
        // We generated the C code from Zig, which contains some warnings
        // that can be safely ignored.
        .flag("-Wno-format")
        .flag("-Wno-format-security")
        .flag("-Wno-unused-but-set-variable")
        .flag("-Wno-unused-const-variable")
        .flag("-Wno-type-limits")
        .flag("-Wno-uninitialized")
        // Ignore warnings when cross-compiling:
        .flag("-Wno-overflow")
        .flag("-Wno-unused-function")
        .flag("-Wno-constant-conversion")
        .flag("-Wno-incompatible-pointer-types")
        // Disable warnings, as we will have lots of them
        .warnings(false)
        .extra_warnings(false)
        .cargo_warnings(false)
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
