use crate::{
    helpers::{clear_dir, get_codspeed_dir},
    prelude::*,
};

use std::{collections::BTreeSet, fs::create_dir_all, rc::Rc};

use cargo::{
    core::{FeatureValue, Package, Verbosity, Workspace},
    ops::{CompileFilter, CompileOptions, Packages},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};
use termcolor::Color;

fn get_compile_options(
    config: &Config,
    features: &Option<Vec<String>>,
    package: &Package,
    benches: Vec<&str>,
    is_root_package: bool,
) -> Result<CompileOptions> {
    let mut compile_opts = CompileOptions::new(config, CompileMode::Build)?;

    // if the package is not the root package, we need to specify the package to build
    if !is_root_package {
        compile_opts.spec = Packages::Packages(vec![package.name().to_string()]);
    }
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
        benches.iter().map(|s| s.to_string()).collect(),
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
    let is_root_package = package_name.is_none();
    let package = match package_name.as_ref() {
        Some(package_name) =>             ws.members()
                .find(|p| p.name().to_string() == *package_name)
                .ok_or_else(|| anyhow!("Package {} not found", package_name))?

        ,
        None => ws.current().map_err(|_| anyhow!("No package found. If working with a workspace please use the -p option to specify a member."))?
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
        Color::White,
    )?;

    let config = ws.config();

    let benches_names = benches.iter().map(|t| t.name()).collect_vec();
    let benches_names_str = benches_names.join(", ");

    ws.config()
        .shell()
        .status_with_color("Building", benches_names_str.clone(), Color::Yellow)?;
    let compile_opts =
        get_compile_options(config, &features, package, benches_names, is_root_package)?;
    let result = cargo::ops::compile(ws, &compile_opts)?;
    let built_benches = result
        .tests
        .into_iter()
        .filter(|u| u.unit.target.is_bench())
        .collect_vec();

    if built_benches.is_empty() {
        bail!(
            "No benchmark target found. \
            Please add a benchmark target to your Cargo.toml"
        );
    }

    ws.config()
        .shell()
        .status_with_color("Built", benches_names_str, Color::Green)?;

    let mut codspeed_target_dir = get_codspeed_dir(ws);
    create_dir_all(&codspeed_target_dir)?;
    if let Some(name) = package_name.as_ref() {
        codspeed_target_dir = codspeed_target_dir.join(name);
        create_dir_all(&codspeed_target_dir)?;
    }
    clear_dir(&codspeed_target_dir)?;

    for built_bench in built_benches.iter() {
        let bench_dest = codspeed_target_dir
            .clone()
            .join(built_bench.unit.target.name());
        std::fs::copy(built_bench.path.clone(), bench_dest)?;
    }

    ws.config().shell().status_with_color(
        "Finished",
        format!("built {} benchmark suite(s)", benches.len()),
        Color::Green,
    )?;

    Ok(())
}
