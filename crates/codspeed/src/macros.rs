#[macro_export]
macro_rules! codspeed_uri {
    ( $name:ident ) => {
        format!("{}::{}", file!(), stringify!($name))
    };
    ( $function:path ) => {
        format!("{}::{}", file!(), stringify!($function))
    };
}

#[macro_export]
macro_rules! codspeed_bench {
    ( $name:ident, $bench_payload:expr) => {
        pub fn $name(codspeed: &mut $crate::codspeed::CodSpeed) {
            let uri = codspeed::codspeed_uri!($name);
            codspeed.start_benchmark(uri.as_str());
            $crate::codspeed::black_box($bench_payload());
            codspeed.end_benchmark();
        }
    };
}

#[macro_export]
macro_rules! codspeed_main {
    ( $( $target:path ),+ $(,)* ) => {
        fn main() {
            $crate::codspeed::display_native_harness();
            let mut codspeed = $crate::codspeed::CodSpeed::new();
            $(
                $target(&mut codspeed);
            )+

        }
    }
}
