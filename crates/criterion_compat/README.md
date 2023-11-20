<div align="center">
<h1>codspeed-criterion-compat</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/codspeed-criterion-compat)](https://crates.io/crates/codspeed-criterion-compat)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-rust)

Criterion.rs compatibility layer for CodSpeed

</div>

## Installation

```sh
cargo add --dev codspeed-criterion-compat
```

## Usage

Let's start with the example from the [Criterion.rs documentation](https://bheisler.github.io/criterion.rs/book/getting_started.html),
creating a benchmark suite for the Fibonacci function (in `benches/my_benchmark.rs`):

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

The last step in creating the Criterion benchmark is to add the new benchmark target in your `Cargo.toml`:

```toml title="Cargo.toml"
[[bench]]
name = "my_benchmark"
harness = false
```

### Plugging CodSpeed

To allow CodSpeed to interact with this suite as well, you simply need to replace
the imports from the `criterion` crate to the `codspeed-criterion-compat` crate:

```diff
- use criterion::{black_box, criterion_group, criterion_main, Criterion};
+ use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};
```

And that's it! You can now run your benchmark suite with `cargo-codspeed`:

```
$ cargo codspeed build
    Finished release [optimized] target(s) in 0.12s
    Finished built 1 benchmark suite(s)

$ cargo codspeed run
   Collected 1 benchmark suite(s) to run
     Running my_benchmark
Using codspeed-criterion-compat v1.0.0 compatibility layer
NOTICE: codspeed is enabled, but no performance measurement will be made since it's running in an unknown environment.
Checked: benches/bencher_example.rs::fib_20 (group: benches)
        Done running bencher_example
    Finished running 1 benchmark suite(s)
```

### Not supported:

- `iter_custom`
- `with_filter`
