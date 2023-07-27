#[macro_export]
macro_rules! criterion_group {
    (name = $name:ident; config = $config:expr; targets = $( $target:path ),+ $(,)*) => {
        pub fn $name(criterion: &mut $crate::Criterion) {
            let mut criterion = &mut criterion.with_patched_measurement($config);
            $(
                criterion.set_current_file($crate::abs_file!());
                criterion.set_macro_group(format!("{}::{}", stringify!($name), stringify!($target)));
                $target(criterion);
            )+
        }
    };
    ($name:ident, $( $target:path ),+ $(,)*) => {
        $crate::criterion_group!{
            name = $name;
            config = $crate::Criterion::default();
            targets = $( $target ),+
        }
    }
}

#[macro_export]
macro_rules! criterion_main {
    ( $( $group:path ),+ $(,)* ) => {
        pub fn main() {
            let mut criterion = $crate::Criterion::new_instrumented();
            $(
                $group(&mut criterion);
            )+
        }
    };
}
