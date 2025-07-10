use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bubble_sort(arr: &mut Vec<i32>) {
    let n = arr.len();
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bubble sort", |b| {
        b.iter(|| {
            let mut data = vec![64, 34, 25, 12, 22, 11, 90];
            bubble_sort(black_box(&mut data));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
