use codspeed_divan_compat::Bencher;

fn fibo(n: u64) -> u64 {
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
fn fibo_50() -> u64 {
    codspeed_divan_compat::black_box(fibo(50))
}

#[codspeed_divan_compat::bench]
fn fibo_10() -> u64 {
    codspeed_divan_compat::black_box(fibo(10))
}

#[codspeed_divan_compat::bench]
fn mut_borrow(bencher: Bencher) {
    let mut bytes = Vec::<i32>::new();

    bencher.bench_local(|| {
        bytes.push(42);
    });
}

// Examples taken from the docs: https://docs.rs/divan/latest/divan/attr.bench.html#consts
mod const_bench {
    const LEN: usize = 42;

    const fn len() -> usize {
        4
    }

    #[codspeed_divan_compat::bench(consts = [1000, LEN, len()])]
    fn init_array<const N: usize>() -> [i32; N] {
        let mut result = [0; N];

        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            result[i] = divan::black_box(i as i32);
        }

        result
    }

    const SIZES: &[usize] = &[1, 10, LEN, len()];
    #[codspeed_divan_compat::bench(consts = SIZES)]
    fn bench_array1<const N: usize>() -> [i32; N] {
        init_array::<N>()
    }

    #[codspeed_divan_compat::bench(consts = SIZES)]
    fn bench_array2<const N: usize>() -> [i32; N] {
        init_array::<N>()
    }
}

fn main() {
    codspeed_divan_compat::main();
}
