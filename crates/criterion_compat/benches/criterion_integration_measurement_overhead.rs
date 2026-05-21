use codspeed_criterion_compat::criterion_main;

mod criterion_integration;

criterion_main! {
    criterion_integration::measurement_overhead::benches,
}
