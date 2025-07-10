fn main() {
    divan::main();
}

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

#[divan::bench]
fn bubble_sort_bench() {
    let mut data = vec![64, 34, 25, 12, 22, 11, 90];
    divan::black_box(bubble_sort(divan::black_box(&mut data)));
}
