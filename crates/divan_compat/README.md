<div align="center">
<h1>codspeed-divan-compat</h1>

[![CI](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/CodSpeedHQ/codspeed-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/codspeed-divan-compat)](https://crates.io/crates/codspeed-divan-compat)
[![Discord](https://img.shields.io/badge/chat%20on-discord-7289da.svg)](https://discord.com/invite/MxpaCfKSqF)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-rust)

Divan compatibility layer for CodSpeed

</div>

## Installation

```sh
cargo add --dev codspeed-divan-compat --rename divan
```

> [!NOTE]
> This will install the `codspeed-divan-compat` crate and rename it to `divan` in your `Cargo.toml`.
> This way, you can keep your existing imports and the compatibility layer will take care of the rest.
>
> Using the compatibility layer won't change the behavior of your benchmark suite and divan will still run it as usual.
>
> If you prefer, you can also install `codspeed-divan-compat` as is and change your imports to use this new crate name.

## Usage

Let's start with the example from the [divan documentation](https://docs.rs/divan/0.1.17/divan/index.html#getting-started),
creating a benchmark suite for the Fibonacci function (in `benches/my_benchmark.rs`):

```rust
fn main() {
    // Run registered benchmarks.
    divan::main();
}

// Register a `fibonacci` function and benchmark it over multiple cases.
#[divan::bench(args = [1, 2, 4, 8, 16, 32])]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}
```

The last step in creating the divan benchmark is to add the new benchmark target in your `Cargo.toml`:

```toml title="Cargo.toml"
[[bench]]
name = "my_benchmark"
harness = false
```

And that's it! You can now run your benchmark suite with `cargo-codspeed`:

```
$ cargo codspeed build
    Finished release [optimized] target(s) in 0.12s
    Finished built 1 benchmark suite(s)

$ cargo codspeed run
   Collected 1 benchmark suite(s) to run
Running my_benchmark
NOTICE: codspeed is enabled, but no performance measurement will be made since it's running in an unknown environment.
Checked: benches/my_benchmark.rs::fibonacci[1]
Checked: benches/my_benchmark.rs::fibonacci[2]
Checked: benches/my_benchmark.rs::fibonacci[4]
Checked: benches/my_benchmark.rs::fibonacci[8]
Checked: benches/my_benchmark.rs::fibonacci[16]
Checked: benches/my_benchmark.rs::fibonacci[32]
Done running my_benchmark
Finished running 1 benchmark suite(s)
```

### Not supported:

- [`divan::bench(crate = xxx)`](https://docs.rs/divan/latest/divan/attr.bench.html#crate): due to how the compatibility layer works internally, we do not plan to support this feature.
- [`divan::bench(consts = xxx)`](https://docs.rs/divan/latest/divan/attr.bench.html#consts): we do not support this feature yet, if you need it don't hesitate to create an issue.
- [`divan::bench_group`](https://docs.rs/divan/latest/divan/attr.bench_group.html): we do not support benchmark grouping yet, if you need it don't hesitate to create an issue.
