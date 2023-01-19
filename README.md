<div align="center">
<h1>codspeed-rust</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-codspeed)](https://crates.io/keywords/codspeed)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)

</div>

This mono-repo contains the integration crates for using CodSpeed in Rust:

- [`cargo-codspeed`](./crates/cargo-codspeed/): A cargo subcommand for running CodSpeed on your project
- [`codspeed-criterion-compat`](./crates/criterion_compat/): Criterion.rs compatibility layer for CodSpeed
- [`codspeed-bencher-compat`](./crates/bencher_compat/): Bencher compatibility layer for CodSpeed
- [`codspeed`](./crates/codspeed/): The core library used to integrate with Codspeed runners
