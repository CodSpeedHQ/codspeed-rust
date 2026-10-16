use tracing::{debug, debug_span, info, info_span, warn};

fn parse(input: &str) -> Vec<u32> {
    let _span = info_span!("parse", input_len = input.len()).entered();
    let values: Vec<u32> = input
        .split(',')
        .filter_map(|field| field.parse().ok())
        .collect();
    info!(count = values.len(), "parsed values");
    values
}

fn sum(values: &[u32]) -> u32 {
    let _span = info_span!("sum").entered();
    let _filtered_out = debug_span!("sum_inner").entered();
    debug!("filtered out by the default filter");
    values.iter().sum()
}

#[codspeed_divan_compat::bench]
fn parse_and_sum() -> u32 {
    let _span = info_span!("parse_and_sum").entered();
    let values = parse("1,2,x,3");
    if values.len() < 4 {
        warn!(skipped = 4 - values.len(), "skipped invalid fields");
    }
    sum(&values)
}

fn main() {
    codspeed_divan_compat::main();
}
