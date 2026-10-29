//! Stands in for the export of the `tracing` spans and events in the builds that don't compile it:
//! only the telemetry builds of `cargo-codspeed` do. Nothing is exported.

use tracing::Subscriber;
use tracing_subscriber::layer::Identity;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// Flushes and shuts down the exporters when dropped.
#[must_use = "dropping the guard shuts the exporters down"]
pub struct TelemetryGuard(());

/// Returns `None`: the layer only exports in the telemetry builds.
pub fn layer<S>() -> Option<(impl Layer<S>, TelemetryGuard)>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    None::<(Identity, TelemetryGuard)>
}

/// Returns `None`: the subscriber is only installed in the telemetry builds.
pub fn init() -> Option<TelemetryGuard> {
    None
}
