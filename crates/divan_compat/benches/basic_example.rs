use codspeed_divan_compat::Bencher;

fn fibo(n: i32) -> i32 {
    let mut a = 0;
    let mut b = 1;

    for _ in 0..n {
        let tmp = a;
        a = b;
        b += tmp;
    }

    a
}

#[codspeed_divan_compat::bench]
fn fibo_500() -> i32 {
    codspeed_divan_compat::black_box(fibo(500))
}

#[codspeed_divan_compat::bench]
fn fibo_10() -> i32 {
    codspeed_divan_compat::black_box(fibo(10))
}

#[codspeed_divan_compat::bench]
fn mut_borrow(bencher: Bencher) {
    let mut bytes = Vec::<i32>::new();

    bencher.bench_local(|| {
        bytes.push(42);
    });
}

fn main() {
    codspeed_divan_compat::main();
}
