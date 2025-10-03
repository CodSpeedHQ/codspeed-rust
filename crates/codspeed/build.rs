use std::{env, path::PathBuf};

fn main() {
    if cfg!(not(target_os = "linux")) {
        // The instrument-hooks library is only supported on Linux.
        return;
    }

    // Compile the C library
    cc::Build::new()
        .file("instrument-hooks/dist/core.c")
        .include("instrument-hooks/includes")
        .flag("-w") // Suppress all warnings
        .compile("instrument_hooks");

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
