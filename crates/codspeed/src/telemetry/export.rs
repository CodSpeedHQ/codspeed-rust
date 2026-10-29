//! The public API, and the subscriber layer and OpenTelemetry providers behind it.

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};
use opentelemetry_sdk::trace::{BatchSpanProcessor, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;
use tracing_log::{AsLog, LogTracer};
use tracing_subscriber::filter::{filter_fn, FilterExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

use super::monotonic::{MonotonicTimeProcessor, MonotonicTimestampLayer};
use super::warmup::{self, warm_up, WarmupFilter};

/// Base URL of the OTLP/HTTP receiver, e.g. `http://localhost:4318`.
const ENDPOINT_ENV: &str = "CODSPEED_OTEL_ENDPOINT";

/// `EnvFilter` directives selecting the exported spans and events. Defaults to `info`.
const FILTER_ENV: &str = "CODSPEED_TELEMETRY_FILTER";

const DEFAULT_FILTER: &str = "info";

/// Ended spans, and log records, queued for export, past which they are dropped. Larger than the
/// SDK default of 2048, which a benchmark emitting on every iteration fills faster than the
/// exporter drains it.
const MAX_QUEUE_SIZE: usize = 1 << 16;

/// Spans or log records sent per export request.
const EXPORT_BATCH_SIZE: usize = 4_096;

/// Spans emitted by the exporter itself would be exported in turn, in an endless loop.
///
/// The OpenTelemetry crates suppress their own, but not those of the HTTP stack under the
/// exporter. These are the directives of the upstream example:
/// <https://github.com/open-telemetry/opentelemetry-rust/blob/20a7b2221c72336f7b7f4c06072a5715bf1dc46a/opentelemetry-otlp/examples/basic-otlp/src/main.rs#L87>
///
/// Directives match targets by prefix, so `hyper` also covers `hyper_util`.
const EXPORTER_FILTER_DIRECTIVES: &[&str] = &["hyper=off", "tonic=off", "h2=off", "reqwest=off"];

#[derive(Debug, Clone)]
pub(super) struct Providers {
    pub(super) tracer: SdkTracerProvider,
    pub(super) logger: SdkLoggerProvider,
}

/// Flushes and shuts down the exporters when dropped.
#[must_use = "dropping the guard shuts the exporters down"]
pub struct TelemetryGuard {
    providers: Providers,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Err(error) = self.providers.tracer.shutdown() {
            eprintln!("Warning: failed to flush OpenTelemetry spans: {error}");
        }
        if let Err(error) = self.providers.logger.shutdown() {
            eprintln!("Warning: failed to flush OpenTelemetry logs: {error}");
        }
    }
}

/// Builds a layer exporting spans and events to the receiver at
/// `CODSPEED_OTEL_ENDPOINT`, to add to a subscriber of your own. Returns `None` when
/// the variable is unset.
///
/// Events are exported as log records carrying the trace and span IDs of the span they are emitted
/// in, rather than as span events.
///
/// Records of the `log` crate reach the subscriber only through a [`LogTracer`], which
/// `tracing-subscriber`'s `init` and `try_init` install with its default `tracing-log` feature.
/// Otherwise, call [`LogTracer::init`].
///
/// Keep the returned guard alive until the benchmarks are done.
pub fn layer<S>() -> Option<(impl Layer<S>, TelemetryGuard)>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let providers = build_providers()?;
    Some((build_layer(&providers), TelemetryGuard { providers }))
}

pub(super) fn build_layer<S>(providers: &Providers) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    // Without a hint, `filter_fn` would lift the max level of the subscriber to `TRACE`.
    let span_filter =
        filter_fn(|metadata| metadata.is_span()).with_max_level_hint(LevelFilter::TRACE);
    let event_filter =
        filter_fn(|metadata| metadata.is_event()).with_max_level_hint(LevelFilter::TRACE);
    let span_layer = MonotonicTimestampLayer::default()
        .and_then(tracing_opentelemetry::layer().with_tracer(providers.tracer.tracer("codspeed")))
        .with_filter(build_filter().and(span_filter));
    let log_layer = OpenTelemetryTracingBridge::new(&providers.logger)
        .with_filter(build_filter().and(event_filter));
    span_layer.and_then(log_layer)
}

/// Installs a global `tracing` subscriber exporting spans and events to the receiver at
/// `CODSPEED_OTEL_ENDPOINT`, and registers its provider as the global OpenTelemetry tracer
/// provider.
///
/// Records of the `log` crate are exported as log records too, unless a `log` logger is already
/// set.
///
/// Returns `None` when the variable is unset or a global subscriber is already set.
pub fn init() -> Option<TelemetryGuard> {
    if tracing::dispatcher::has_been_set() {
        return None;
    }

    let (layer, guard) = layer()?;
    let subscriber = tracing_subscriber::registry().with(layer);
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        return None;
    }
    // Set after the subscriber, so that `log` skips the records above its max level.
    let _ = LogTracer::builder()
        .with_max_level(LevelFilter::current().as_log())
        .init();
    opentelemetry::global::set_tracer_provider(guard.providers.tracer.clone());
    warm_up();
    Some(guard)
}

fn build_providers() -> Option<Providers> {
    let endpoint = std::env::var(ENDPOINT_ENV)
        .ok()
        .filter(|endpoint| !endpoint.is_empty())?;
    let endpoint = endpoint.trim_end_matches('/');

    let exporters = SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{endpoint}/v1/traces"))
        .build()
        .and_then(|span_exporter| {
            let log_exporter = LogExporter::builder()
                .with_http()
                .with_endpoint(format!("{endpoint}/v1/logs"))
                .build()?;
            Ok((span_exporter, log_exporter))
        });
    let (span_exporter, log_exporter) = match exporters {
        Ok(exporters) => exporters,
        Err(error) => {
            eprintln!("Warning: failed to build the OpenTelemetry exporters: {error}");
            return None;
        }
    };

    let resource = Resource::builder()
        .with_service_name(service_name())
        .build();
    let span_processor = BatchSpanProcessor::builder(span_exporter)
        .with_batch_config(
            opentelemetry_sdk::trace::BatchConfigBuilder::default()
                .with_max_queue_size(MAX_QUEUE_SIZE)
                .with_max_export_batch_size(EXPORT_BATCH_SIZE)
                .build(),
        )
        .build();
    let log_processor = BatchLogProcessor::builder(log_exporter)
        .with_batch_config(
            opentelemetry_sdk::logs::BatchConfigBuilder::default()
                .with_max_queue_size(MAX_QUEUE_SIZE)
                .with_max_export_batch_size(EXPORT_BATCH_SIZE)
                .build(),
        )
        .build();
    Some(Providers {
        tracer: SdkTracerProvider::builder()
            .with_span_processor(WarmupFilter(span_processor))
            .with_resource(resource.clone())
            .build(),
        logger: SdkLoggerProvider::builder()
            .with_log_processor(MonotonicTimeProcessor)
            .with_log_processor(WarmupFilter(log_processor))
            .with_resource(resource)
            .build(),
    })
}

fn build_filter() -> EnvFilter {
    let user_directives = std::env::var(FILTER_ENV).unwrap_or_else(|_| DEFAULT_FILTER.to_owned());
    let warmup_directive = warmup::filter_directive();
    let directives = [user_directives.as_str(), warmup_directive.as_str()]
        .into_iter()
        .chain(EXPORTER_FILTER_DIRECTIVES.iter().copied())
        .collect::<Vec<_>>()
        .join(",");
    EnvFilter::builder().parse_lossy(directives)
}

/// The benchmark executable name, without the hash cargo appends to it.
fn service_name() -> String {
    let file_stem = std::env::current_exe().ok().and_then(|exe| {
        exe.file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
    });
    match file_stem {
        Some(stem) => match stem.rsplit_once('-') {
            Some((name, hash)) if is_cargo_hash(hash) => name.to_owned(),
            _ => stem,
        },
        None => "codspeed-benchmark".to_owned(),
    }
}

fn is_cargo_hash(s: &str) -> bool {
    s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit())
}
