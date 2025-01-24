fn main() {
    if std::env::var("CARGO_FEATURE_VENDORED_OPENSSL").is_ok() {
        println!("cargo:warning=The `vendored-openssl` feature is deprecated and no longer does anything. Please remove it from your Cargo.toml.");
    }
}
