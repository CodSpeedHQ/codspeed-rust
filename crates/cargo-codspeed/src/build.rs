use crate::{
    helpers::{clear_dir, get_codspeed_dir},
    prelude::*,
};

use std::{collections::BTreeSet, fs::create_dir_all, rc::Rc};

use anstyle::{AnsiColor, Style};
use cargo::{
    core::{FeatureValue, Package, Verbosity, Workspace},
    ops::{CompileFilter, CompileOptions, Packages},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};

fn get_compile_options(
    config: &Config,
    features: &Option<Vec<String>>,
    package: &Package,
    bench: &str,
) -> Result<CompileOptions> {
    let mut compile_opts = CompileOptions::new(config, CompileMode::Build)?;
    compile_opts.spec = Packages::Packages(vec![package.name().to_string()]);
    if let Some(features) = features {
        compile_opts.cli_features.features = Rc::new(
            features
                .iter()
                .map(|s| FeatureValue::Feature(InternedString::new(s.as_str())))
                .collect::<BTreeSet<FeatureValue>>(),
        );
    }
    compile_opts.build_config.requested_profile = "release".into();
    compile_opts.filter = CompileFilter::from_raw_arguments(
        false,
        vec![],
        false,
        vec![],
        false,
        vec![],
        false,
        vec![bench.into()],
        false,
        false,
    );
    Ok(compile_opts)
}

pub fn build_benches(
    ws: &Workspace,
    selected_benches: Option<Vec<String>>,
    package_name: Option<String>,
    features: Option<Vec<String>>,
) -> Result<()> {
    let package = match package_name.as_ref() {
        Some(package_name) => ws
            .members()
            .find(|p| p.name().to_string() == *package_name)
            .ok_or_else(|| anyhow!("Package {} not found", package_name))?,
        None => ws.current().map_err(|_| anyhow!("No package found. If working with a workspace please use the -p option to specify a member."))?,
    };

    let all_benches = package
        .targets()
        .iter()
        .filter(|t| t.is_bench())
        .collect_vec();

    let all_benches_count = all_benches.len();

    let benches = if let Some(selected_benches) = selected_benches {
        all_benches
            .into_iter()
            .filter(|t| selected_benches.contains(&t.name().to_string()))
            .collect_vec()
    } else {
        all_benches
    };

    ws.config().shell().set_verbosity(Verbosity::Normal);
    ws.config().shell().status_with_color(
        "Collected",
        format!(
            "{} benchmark suite(s) to build{}",
            benches.len(),
            if all_benches_count > benches.len() {
                format!(" ({} filtered out)", all_benches_count - benches.len())
            } else {
                "".to_string()
            }
        ),
        &Style::new().fg_color(Some(AnsiColor::White.into())),
    )?;

    let config = ws.config();
    let mut built_benches = vec![];
    for bench in benches {
        ws.config().shell().status_with_color(
            "Building",
            bench.name(),
            &Style::new().fg_color(Some(AnsiColor::Yellow.into())),
        )?;
        let compile_opts = get_compile_options(config, &features, package, bench.name())?;
        let result = cargo::ops::compile(ws, &compile_opts)?;
        let built_targets = result
            .tests
            .into_iter()
            .filter(|u| u.unit.target.is_bench())
            .collect_vec();
        if let Some(built_bench) = built_targets.into_iter().next() {
            built_benches.push(built_bench);
        } else {
            bail!("No benchmark target found.")
        }
        ws.config().shell().status_with_color(
            "Built",
            bench.name(),
            &Style::new().fg_color(Some(AnsiColor::Green.into())),
        )?;
    }

    if built_benches.is_empty() {
        bail!(
            "No benchmark target found. \
            Please add a benchmark target to your Cargo.toml"
        );
    }

    let mut codspeed_target_dir = get_codspeed_dir(ws);
    create_dir_all(&codspeed_target_dir)?;
    if let Some(name) = package_name.as_ref() {
        codspeed_target_dir = codspeed_target_dir.join(name);
        create_dir_all(&codspeed_target_dir)?;
    }
    clear_dir(&codspeed_target_dir)?;

    for bench in built_benches.iter() {
        let bench_dest = codspeed_target_dir.clone().join(bench.unit.target.name());
        std::fs::copy(bench.path.clone(), bench_dest)?;
    }

    ws.config().shell().status_with_color(
        "Finished",
        format!("built {} benchmark suite(s)", built_benches.len()),
        &Style::new().fg_color(Some(AnsiColor::Green.into())),
    )?;

    Ok(())
}
