use codspeed_criterion_compat::criterion_main;

mod criterion_integration;

criterion_main! {
    criterion_integration::compare_functions::fibonaccis,
    // criterion_integration::external_process::benches, FIXME: Currently doesn't work
    criterion_integration::iter_with_large_drop::benches,
    criterion_integration::iter_with_large_setup::benches,
    criterion_integration::iter_with_setup::benches,
    criterion_integration::with_inputs::benches,
    criterion_integration::special_characters::benches,
    criterion_integration::measurement_overhead::benches,
    criterion_integration::custom_measurement::benches,
    criterion_integration::sampling_mode::benches,
    criterion_integration::async_measurement_overhead::benches,
}
