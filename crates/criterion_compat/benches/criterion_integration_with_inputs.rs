use codspeed_criterion_compat::criterion_main;

mod criterion_integration;

criterion_main! {
    criterion_integration::with_inputs::benches,
}
