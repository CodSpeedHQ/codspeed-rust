fn main() {
    println!("cargo:rustc-check-cfg=cfg(use_instrument_hooks)");

    println!("cargo:rerun-if-changed=instrument-hooks/dist/core.c");
    println!("cargo:rerun-if-changed=instrument-hooks/includes/core.h");
    println!("cargo:rerun-if-changed=build.rs");

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
