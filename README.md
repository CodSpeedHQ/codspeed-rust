# codspeed-rust

This mono-repo contains the integration crates for using CodSpeed in Rust:

- [`cargo-codspeed`](./crates/cargo-codspeed/): A cargo subcommand for running CodSpeed on your project
- [`codspeed-criterion-compat`](./crates/criterion_compat/): Criterion.rs compatibility layer for CodSpeed
- [`codspeed-bencher-compat`](./crates/bencher_compat/): Bencher compatibility layer for CodSpeed
- [`codspeed`](./crates/codspeed/): The core library used to integrate with Codspeed runners
