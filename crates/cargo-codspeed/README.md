<div align="center">
<h1>cargo-codspeed</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-codspeed)](https://crates.io/crates/cargo-codspeed)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-rust)

A cargo subcommand for running CodSpeed on your project

</div>

## Installation

### With `cargo`

```bash
cargo install cargo-codspeed --locked
```

### With `cargo-binstall`(recommended in CI)

[`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) enables you to install binaries directly without having to build from the source(with `cargo install`) every time.

If you don't have installed yet, you can install it with:

```bash
cargo install cargo-binstall
```

You can then install `cargo-codspeed` with:

```bash
cargo binstall cargo-codspeed
```

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

### Running benchmarks with details

Use the `--details` flag to see timing information for each benchmark:

```bash
cargo codspeed run --details
```

This will show execution times for each benchmark:
```
  Checked: benches/example.rs::fibonacci (5.6 us)
  Checked: benches/example.rs::factorial (368 ns)
```

The output also shows the total number of benchmarks executed:
```
Done running benchmark_suite (5 benchmarks)
Finished running 2 benchmark suite(s) (10 benchmarks total)
```

## Development

### Troubleshooting

- Build error on MacOS: `ld: library 'git2' not found`

  ```
  brew install libgit2
  ```
