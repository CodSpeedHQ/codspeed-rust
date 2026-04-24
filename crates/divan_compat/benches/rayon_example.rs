use std::hint::black_box;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

#[inline(never)]
fn sum_primes(limit: u32) -> u64 {
    let mut sum = 0u64;
    for n in 2..limit {
        let mut is_prime = true;
        let mut i = 2;
        while i * i <= n {
            if n % i == 0 {
                is_prime = false;
                break;
            }
            i += 1;
        }
        if is_prime {
            sum += n as u64;
        }
    }
    sum
}

#[inline(never)]
fn fibo(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibo(n - 1) + fibo(n - 2)
    }
}

#[inline(never)]
fn hash_rounds(seed: u64, rounds: u32) -> u64 {
    let mut state = seed;
    for _ in 0..rounds {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state ^= state >> 33;
    }
    state
}

#[inline(never)]
fn matmul(size: usize) -> f64 {
    let a: Vec<f64> = (0..size * size).map(|i| (i as f64) * 0.001).collect();
    let b: Vec<f64> = (0..size * size).map(|i| (i as f64) * 0.002).collect();
    let mut c = vec![0.0f64; size * size];
    for i in 0..size {
        for k in 0..size {
            let aik = a[i * size + k];
            for j in 0..size {
                c[i * size + j] += aik * b[k * size + j];
            }
        }
    }
    c.iter().sum()
}

enum Task {
    Primes(u32),
    Fib(u64),
    Hash(u64, u32),
    MatMul(usize),
}

#[inline(never)]
fn run_task(task: &Task) -> u64 {
    match *task {
        Task::Primes(n) => sum_primes(n),
        Task::Fib(n) => fibo(n),
        Task::Hash(seed, rounds) => hash_rounds(seed, rounds),
        Task::MatMul(size) => matmul(size).to_bits(),
    }
}

fn workload() -> Vec<Task> {
    vec![
        Task::Primes(20_000),
        Task::Fib(28),
        Task::Hash(0xDEADBEEF, 2_000_000),
        Task::MatMul(96),
        Task::Primes(25_000),
        Task::Fib(29),
        Task::Hash(0xCAFEBABE, 2_500_000),
        Task::MatMul(80),
    ]
}

#[codspeed_divan_compat::bench]
fn rayon_mixed_workload() -> u64 {
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let tasks = workload();
    pool.install(|| {
        tasks
            .par_iter()
            .map(|t| black_box(run_task(black_box(t))))
            .reduce(|| 0u64, |a, b| a.wrapping_add(b))
    })
}

fn main() {
    codspeed_divan_compat::main();
}
