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

fn main() {
    codspeed_divan_compat::main();
}
