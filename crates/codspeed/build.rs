use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=instrument-hooks/dist/core.c");
    println!("cargo:rerun-if-changed=instrument-hooks/includes/core.h");
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(not(target_os = "linux")) {
        // The instrument-hooks library is only supported on Linux.
        return;
    }

    let mut build = cc::Build::new();
    build
        .flag("-std=c11")
        .file("instrument-hooks/dist/core.c")
        .include("instrument-hooks/includes")
        .warnings(false)
        .extra_warnings(false)
        .cargo_warnings(false);

    let result = build.try_compile("instrument_hooks");
    if let Err(e) = result {
        let compiler = build.try_get_compiler().expect("Failed to get C compiler");

        eprintln!("\n\nERROR: Failed to compile instrument-hooks native library with cc-rs. Ensure you have an up-to-date C compiler installed.");
        eprintln!("Compiler information: {:?}", compiler);
        eprintln!("Compilation error: {e}");

        std::process::exit(1);
    }

    let bindings = bindgen::Builder::default()
        .header("instrument-hooks/includes/core.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
