use codspeed::{
    codspeed::{black_box, CodSpeed, WARMUP_RUNS},
    utils::{get_formated_function_path, get_git_relative_path},
};

pub struct Bencher {
    pub bytes: u64,
    codspeed: CodSpeed,
    current_file: String,
    current_bench_path: String,
}

impl Bencher {
    pub fn set_current_file(&mut self, file: impl Into<String>) {
        self.current_file = file.into();
    }

    pub fn set_current_bench_path(&mut self, bench: impl Into<String>) {
        self.current_bench_path = bench.into();
    }

    pub fn push_group(&mut self, group: &str) {
        self.codspeed.push_group(group);
    }

    pub fn pop_group(&mut self) {
        self.codspeed.pop_group();
    }

    pub fn iter<T, F>(&mut self, mut inner: F)
    where
        F: FnMut() -> T,
    {
        let file = get_git_relative_path(self.current_file.as_str());
        let bench_path = get_formated_function_path(self.current_bench_path.as_str());
        let uri = format!("{}::{}", file.to_string_lossy(), bench_path);
        for _ in 0..WARMUP_RUNS {
            black_box(inner());
        }
        self.codspeed.start_benchmark(uri.as_str());
        black_box(inner());
        self.codspeed.end_benchmark();
    }
}

impl Default for Bencher {
    fn default() -> Self {
        println!(
            "Harness: codspeed-bencher-compat v{}",
            env!("CARGO_PKG_VERSION"),
        );
        Bencher {
            bytes: 0,
            codspeed: CodSpeed::new(),
            current_file: String::new(),
            current_bench_path: String::new(),
        }
    }
}

#[macro_export]
macro_rules! benchmark_group {
    ($group_name:ident, $( $function:path ),+ $(,)*) => {
        pub fn $group_name(bencher: &mut $crate::Bencher) {
            bencher.push_group(stringify!($group_name));
            $(
                bencher.set_current_file($crate::abs_file!());
                bencher.set_current_bench_path(stringify!($function));
                $function(bencher);
            )+
            bencher.pop_group();
        }
    };
    ($group_name:ident, $($function:path,)+) => {
        benchmark_group!($group_name, $($function),+);
    };
}

#[macro_export]
macro_rules! benchmark_main {
    ($($group_name:path),+) => {
        pub fn main() {
            let mut bencher = $crate::Bencher::default();
            $(
                $group_name(&mut bencher);
            )+
        }
    };
    ($($group_name:path,)+) => {
        benchmark_main!($($group_name),+);
    };
}
