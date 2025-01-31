#[macro_export]
macro_rules! criterion_group {
    (name = $name:ident; config = $config:expr; targets = $( $target:path ),+ $(,)*) => {
        pub fn $name() {
            let mut criterion: $crate::Criterion<_> = $config
                .configure_from_args();
            $(
                // ### start new code
                eprintln!("file: {}, name: {}, target: {}", $crate::abs_file!(), stringify!($name), stringify!($target));
                // ### end new code
                $target(&mut criterion);
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
