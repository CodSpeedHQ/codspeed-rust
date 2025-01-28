pub use codspeed::codspeed_uri;

#[cfg(not(codspeed))]
mod compat_divan {
    pub use divan::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_divan {
    pub use divan::black_box;

    pub use codspeed_divan_compat_macros::bench_compat as bench;
    // Important: Keep in sync with the name used in the compat macro
    pub use divan::bench as bench_original;

    mod compat;
    pub use compat::*;
}

pub use compat_divan::*;
