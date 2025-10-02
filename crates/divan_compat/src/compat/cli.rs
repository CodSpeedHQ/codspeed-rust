use clap::{Arg, ArgAction, Command};

pub(crate) fn command() -> Command {
    fn option(name: &'static str) -> Arg {
        Arg::new(name).long(name)
    }

    fn flag(name: &'static str) -> Arg {
        option(name).action(ArgAction::SetTrue)
    }

    Command::new("divan")
        .arg(
            Arg::new("filter")
                .value_name("FILTER")
                .help("Only run benchmarks whose names match this pattern")
                .action(ArgAction::Append),
        )
        .arg(flag("exact").help("Filter benchmarks by exact name rather than by pattern"))
}
