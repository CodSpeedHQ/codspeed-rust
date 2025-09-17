fn fibo(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibo(n - 1) + fibo(n - 2)
    }
}

#[codspeed_divan_compat::bench(args = [30, 31, 32])]
fn fib_in_thread(n: usize) {
    let handle = std::thread::spawn(move || codspeed_divan_compat::black_box(fibo(n as u64)));
    handle.join().unwrap();
}

#[codspeed_divan_compat::bench(args = [30, 31, 32])]
fn fib_in_thread_bench(bencher: codspeed_divan_compat::Bencher, n: usize) {
    bencher.bench(|| {
        let handle = std::thread::spawn(move || codspeed_divan_compat::black_box(fibo(n as u64)));
        handle.join().unwrap()
    })
}

#[codspeed_divan_compat::bench(args = [30, 31, 32])]
fn fib_in_thread_bench_local(bencher: codspeed_divan_compat::Bencher, n: usize) {
    bencher.bench_local(|| {
        let handle = std::thread::spawn(move || codspeed_divan_compat::black_box(fibo(n as u64)));
        handle.join().unwrap()
    })
}

fn main() {
    codspeed_divan_compat::main();
}
