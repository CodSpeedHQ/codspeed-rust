use crate::{
    helpers::{clear_dir, get_codspeed_target_dir, style},
    prelude::*,
};

use std::{collections::BTreeSet, fs::create_dir_all, rc::Rc};

use cargo::{
    core::{FeatureValue, Package, Target, Verbosity, Workspace},
    ops::{CompileFilter, CompileOptions, Packages},
    util::{command_prelude::CompileMode, interning::InternedString},
    Config,
};

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

struct BenchToBuild<'a> {
    package: &'a Package,
    target: &'a Target,
}

pub fn build_benches(
    ws: &Workspace,
    selected_benches: Option<Vec<String>>,
    packages: Packages,
    features: Option<Vec<String>>,
) -> Result<()> {
    let packages_to_build = packages.get_packages(ws)?;
    let mut benches_to_build = vec![];
    for package in packages_to_build.iter() {
        let benches = package
            .targets()
            .iter()
            .filter(|t| t.is_bench())
            .collect_vec();
        benches_to_build.extend(
            benches
                .into_iter()
                .map(|t| BenchToBuild { package, target: t }),
        );
    }

    let all_benches_count = benches_to_build.len();

    let benches_to_build = if let Some(selected_benches) = selected_benches {
        benches_to_build
            .into_iter()
            .filter(|t| selected_benches.contains(&t.target.name().to_string()))
            .collect_vec()
    } else {
        benches_to_build
    };

    let actual_benches_count = benches_to_build.len();

    ws.config().shell().set_verbosity(Verbosity::Normal);
    ws.config().shell().status_with_color(
        "Collected",
        format!(
            "{} benchmark suite(s) to build{}",
            benches_to_build.len(),
            if all_benches_count > actual_benches_count {
                format!(
                    " ({} filtered out)",
                    all_benches_count - actual_benches_count
                )
            } else {
                "".to_string()
            }
        ),
        &style::TITLE,
    )?;

    let config = ws.config();
    let codspeed_root_target_dir = get_codspeed_target_dir(ws);
    // Create and clear packages target directories
    for package in packages_to_build.iter() {
        let package_target_dir = codspeed_root_target_dir.join(package.name());
        create_dir_all(&package_target_dir)?;
        clear_dir(&package_target_dir)?;
    }
    let mut built_benches = 0;
    for bench in benches_to_build.iter() {
        ws.config().shell().status_with_color(
            "Building",
            format!("{} {}", bench.package.name(), bench.target.name()),
            &style::ACTIVE,
        )?;
        let is_root_package = ws.current_opt().map_or(false, |p| p == bench.package);
        let benches_names = vec![bench.target.name()];
        let compile_opts = get_compile_options(
            config,
            &features,
            bench.package,
            benches_names,
            is_root_package,
        )?;
        let result = cargo::ops::compile(ws, &compile_opts)?;
        let built_units = result
            .tests
            .into_iter()
            .filter(|u| u.unit.target.is_bench())
            .collect_vec();
        if built_units.is_empty() {
            continue;
        }
        built_benches += 1;
        let codspeed_target_dir = codspeed_root_target_dir.join(bench.package.name());
        for built_unit in built_units.iter() {
            let bench_dest = codspeed_target_dir
                .clone()
                .join(built_unit.unit.target.name());
            std::fs::copy(built_unit.path.clone(), bench_dest)?;
        }
    }

    if built_benches == 0 {
        bail!(
            "No benchmark target found. \
            Please add a benchmark target to your Cargo.toml"
        );
    }
    ws.config().shell().status_with_color(
        "Finished",
        format!("built {} benchmark suite(s)", built_benches),
        &style::SUCCESS,
    )?;

    Ok(())
}
