use codspeed_divan_compat as divan;

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

#[divan::bench]
fn fibo_500() -> i32 {
    divan::black_box(fibo(500))
}

#[divan::bench]
fn fibo_100() -> i32 {
    divan::black_box(fibo(10))
}

fn main() {
    // Run `add` benchmark:
    divan::main();
}
