<div align="center">
<h1>cargo-codspeed</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-codspeed)](https://crates.io/crates/cargo-codspeed)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)

A cargo subcommand for running CodSpeed on your project

</div>

## Installation

```bash
cargo install cargo-codspeed
```

Feature `vendored-openssl` can be used to statically link with openssl with `cargo install cargo-codspeed --features vendored-openssl`.

## Usage

```
Usage: cargo codspeed <COMMAND>

Commands:
  build  Build the benchmarks
  run    Run the previously built benchmarks

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
