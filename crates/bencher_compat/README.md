<div align="center">
<h1>codspeed-bencher-compat</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/codspeed-bencher-compat)](https://crates.io/crates/codspeed-bencher-compat)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-rust)

Bencher compatibility layer for CodSpeed

</div>

## Installation

```sh
cargo add --dev codspeed-bencher-compat --rename bencher
```

> [!NOTE]
> This will install the `codspeed-bencher-compat` crate and rename it to `bencher` in your `Cargo.toml`.
> This way, you can keep your existing imports and the compatibility layer will take care of the rest.
>
> Using the compatibility layer won't change the behavior of your benchmark suite and Bencher will still run it as usual.
>
> If you prefer, you can also install `codspeed-bencher-compat` as is and change your imports to use this new crate name.

## Usage

Let's start with the example from the [Bencher documentation](https://docs.rs/bencher/latest/bencher/),
creating a benchmark suite for 2 simple functions (in `benches/example.rs`):

```rust
use bencher::{benchmark_group, benchmark_main, Bencher};

fn a(bench: &mut Bencher) {
    bench.iter(|| {
        (0..1000).fold(0, |x, y| x + y)
    })
}

fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| {
        vec![0u8; N]
    });

    bench.bytes = N as u64;
}

benchmark_group!(benches, a, b);
benchmark_main!(benches);
```

The last step in creating the Bencher benchmark is to add the new benchmark target in your `Cargo.toml`:

```toml title="Cargo.toml"
[[bench]]
name = "example"
harness = false
```

And that's it! You can now run your benchmark suite with CodSpeed:

```
$ cargo codspeed build
    Finished release [optimized] target(s) in 0.12s
    Finished built 1 benchmark suite(s)

$ cargo codspeed run
   Collected 1 benchmark suite(s) to run
     Running example
Using codspeed-bencher-compat v1.0.0 compatibility layer
NOTICE: codspeed is enabled, but no performance measurement will
be made since it's running in an unknown environment.
Checked: benches/example.rs::a (group: benches)
Checked: benches/example.rs::b (group: benches)
        Done running bencher_example
    Finished running 1 benchmark suite(s)
```
