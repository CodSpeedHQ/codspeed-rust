//! Example benchmarks demonstrating divan counter usage
//!
//! This file shows how to use different types of counters with divan:
//! - BytesCount: for measuring throughput in bytes
//! - ItemsCount: for counting processed items
//! - CharsCount: for counting processed characters
//! - CyclesCount: for counting processing cycles

use divan::{counter::*, AllocProfiler, Bencher};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

/// Example data for benchmarks
const SAMPLE_DATA: &[i32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const SAMPLE_TEXT: &str = "Hello, world! This is a sample string for benchmarking.";

mod bytes_counter_examples {
    use super::*;

    #[divan::bench]
    fn vec_copy_with_bytes_counter(bencher: Bencher) {
        let data = SAMPLE_DATA;
        let bytes = BytesCount::of_slice(data);

        bencher
            .counter(bytes)
            .bench(|| -> Vec<i32> { divan::black_box(data).to_vec() });
    }

    #[divan::bench]
    fn string_copy_with_bytes_counter(bencher: Bencher) {
        let text = SAMPLE_TEXT;
        let bytes = BytesCount::of_str(text);

        bencher
            .counter(bytes)
            .bench(|| -> String { divan::black_box(text).to_owned() });
    }

    #[divan::bench]
    fn slice_into_vec_with_bytes(bencher: Bencher) {
        let ints = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let bytes = BytesCount::of_slice(ints);

        bencher
            .counter(bytes)
            .bench(|| -> Vec<i32> { divan::black_box(ints).into() });
    }
}

mod items_counter_examples {
    use super::*;

    #[divan::bench]
    fn process_items_with_counter(bencher: Bencher) {
        let data = SAMPLE_DATA;
        let items = ItemsCount::new(data.len());

        bencher
            .counter(items)
            .bench(|| -> Vec<i32> { divan::black_box(data).iter().map(|x| x * 2).collect() });
    }

    #[divan::bench]
    fn filter_items_with_counter(bencher: Bencher) {
        let data = (1..=100).collect::<Vec<_>>();
        let items = ItemsCount::new(data.len());

        bencher.counter(items).bench(|| -> Vec<i32> {
            divan::black_box(&data)
                .iter()
                .filter(|&&x| x % 2 == 0)
                .copied()
                .collect()
        });
    }
}

mod chars_counter_examples {
    use super::*;

    #[divan::bench]
    fn count_chars_in_string(bencher: Bencher) {
        let text = SAMPLE_TEXT;
        let chars = CharsCount::of_str(text);

        bencher
            .counter(chars)
            .bench(|| -> usize { divan::black_box(text).chars().count() });
    }

    #[divan::bench]
    fn uppercase_chars_with_counter(bencher: Bencher) {
        let text = "hello world with unicode: café naïve résumé";
        let chars = CharsCount::of_str(text);

        bencher
            .counter(chars)
            .bench(|| -> String { divan::black_box(text).to_uppercase() });
    }
}

mod cycles_counter_examples {
    use super::*;

    #[divan::bench]
    fn simulated_processing_cycles(bencher: Bencher) {
        // Simulate processing 1000 "cycles" of work
        let cycles = CyclesCount::new(1000u32);

        bencher.counter(cycles).bench(|| {
            // Simulate some work that processes 1000 cycles
            let mut sum = 0u64;
            for i in 0..1000 {
                sum = sum.wrapping_add(divan::black_box(i));
            }
            sum
        });
    }

    #[divan::bench]
    fn hash_computation_cycles(bencher: Bencher) {
        let data = SAMPLE_DATA;
        // Treat each hash operation as processing N cycles where N = data length
        let cycles = CyclesCount::new(data.len());

        bencher.counter(cycles).bench(|| -> u64 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            divan::black_box(data).hash(&mut hasher);
            hasher.finish()
        });
    }
}

mod multiple_counters_examples {
    use super::*;

    #[divan::bench(counters = [BytesCount::of_slice(SAMPLE_DATA), ItemsCount::new(SAMPLE_DATA.len())])]
    fn process_with_multiple_counters() -> Vec<i32> {
        SAMPLE_DATA.iter().map(|x| x * x).collect()
    }

    #[divan::bench]
    fn string_processing_multi_counter(bencher: Bencher) {
        let text = "Processing this text with multiple counters";

        bencher
            .counter(BytesCount::of_str(text))
            .counter(CharsCount::of_str(text))
            .bench(|| -> Vec<char> { divan::black_box(text).chars().collect() });
    }
}
