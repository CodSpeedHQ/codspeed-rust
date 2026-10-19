#[divan::bench]
fn traced() -> u32 {
    let _span = tracing::info_span!("traced").entered();
    divan::black_box(1) + 1
}

fn main() {
    eprintln!(
        "tracing static max level: {}",
        tracing::level_filters::STATIC_MAX_LEVEL
    );
    divan::main();
}
