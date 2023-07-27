#[cfg(not(codspeed))]
mod compat_bencher {
    pub use bencher::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_bencher {
    pub use codspeed::abs_file;

    mod compat;
    pub use compat::*;
}

pub use compat_bencher::*;
