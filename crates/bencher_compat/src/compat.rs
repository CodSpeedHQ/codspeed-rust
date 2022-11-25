use codspeed::codspeed::{black_box, CodSpeed};

pub struct Bencher {
    pub bytes: u64,
    codspeed: CodSpeed,
    current_uri: String,
}

impl Bencher {
    pub fn set_current_uri(&mut self, uri: String) {
        self.current_uri = uri;
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
        let uri = self.current_uri.as_str();
        self.codspeed.start_benchmark(uri);
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
            current_uri: String::new(),
        }
    }
}

#[macro_export]
macro_rules! benchmark_group {
    ($group_name:ident, $($function:path),+) => {
        pub fn $group_name(bencher: &mut $crate::Bencher) {
            bencher.push_group(stringify!($group_name));
            $(
                bencher.set_current_uri($crate::codspeed_uri!($function));
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
