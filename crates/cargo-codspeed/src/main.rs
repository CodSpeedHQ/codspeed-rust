mod app;
mod build;
mod helpers;
mod prelude;
mod run;

use crate::prelude::*;
use std::{env::args_os, process::exit};

fn main() {
    let mut args_vec = args_os().collect_vec();

    if args_vec.len() >= 2 && args_vec[1] == "codspeed" {
        args_vec[1] = "cargo codspeed".into();
        args_vec = args_vec.into_iter().skip(1).collect_vec();
    };

    if let Err(e) = app::run(args_vec.into_iter()) {
        eprintln!("{}", e);
        exit(1);
    }
}
