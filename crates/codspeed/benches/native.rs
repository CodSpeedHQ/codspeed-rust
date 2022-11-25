use std::collections::HashMap;

use codspeed::{codspeed::black_box, codspeed_bench, codspeed_main};

pub fn fibonacci_recursive_cached(n: u64) -> u64 {
    fn inner(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
        match n {
            0 | 1 => 1,
            _ => {
                if cache.contains_key(&n) {
                    *cache.get(&n).unwrap()
                } else {
                    let result = inner(n - 1, cache) + inner(n - 2, cache);
                    cache.insert(n, result);
                    result
                }
            }
        }
    }
    let mut cache = HashMap::new();
    inner(n, &mut cache)
}

pub fn fibonacci_recursive(n: i32) -> u64 {
    match n {
        0 | 1 => 1,
        _ => fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2),
    }
}

pub fn fibonacci_iterative(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut sum = 0;
    let mut last = 0;
    let mut curr = 1;
    for _i in 1..n {
        sum = last + curr;
        last = curr;
        curr = sum;
    }
    sum
}

codspeed_bench!(fibo_recursive, || fibonacci_recursive(black_box(10)));
codspeed_bench!(fibo_iterative, || fibonacci_iterative(black_box(10)));
codspeed_bench!(fibo_recursive_cached, || fibonacci_recursive_cached(
    black_box(10)
));
codspeed_main!(fibo_recursive, fibo_iterative, fibo_recursive_cached);
