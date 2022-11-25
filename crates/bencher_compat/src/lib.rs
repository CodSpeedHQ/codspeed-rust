pub use codspeed::codspeed_uri;

#[cfg(not(codspeed))]
mod compat_bencher {
    pub use bencher::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_bencher {
    mod compat;
    pub use compat::*;
}

pub use compat_bencher::*;
