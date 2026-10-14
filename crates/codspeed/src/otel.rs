//! Export of the `tracing` spans and events emitted by benchmarks over OTLP/HTTP.
//!
//! Everything is a no-op unless [`ENDPOINT_ENV`] is set.

use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use opentelemetry::trace::{Span as _, TracerProvider as _};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{SdkTracerProvider, Span, SpanData, SpanProcessor};
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, EnvFilter, Layer};

/// Base URL of the OTLP/HTTP receiver, e.g. `http://localhost:4318`.
pub const ENDPOINT_ENV: &str = "CODSPEED_OTEL_ENDPOINT";

/// `EnvFilter` directives selecting the exported spans and events. Defaults to `info`.
pub const FILTER_ENV: &str = "CODSPEED_OTEL_FILTER";

const DEFAULT_FILTER: &str = "info";

/// Span attribute holding the URI of the benchmark running when the span started.
pub const BENCHMARK_URI_ATTRIBUTE: &str = "codspeed.benchmark.uri";

static CURRENT_BENCHMARK: RwLock<Option<Arc<str>>> = RwLock::new(None);

/// Spans emitted by the exporter itself would be exported in turn, in an endless loop.
const EXPORTER_FILTER_DIRECTIVES: &[&str] = &[
    "h2=off",
    "hyper=off",
    "hyper_util=off",
    "opentelemetry=off",
    "opentelemetry_otlp=off",
    "opentelemetry_sdk=off",
    "opentelemetry_http=off",
    "reqwest=off",
    "tower=off",
];

/// Flushes and shuts down the exporter when dropped.
#[must_use = "dropping the guard shuts the exporter down"]
pub struct OtelGuard {
    provider: SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(error) = self.provider.shutdown() {
            eprintln!("Warning: failed to flush OpenTelemetry spans: {error}");
        }
    }
}

/// Builds a layer exporting spans and events to the receiver at
/// [`ENDPOINT_ENV`], to add to a subscriber of your own. Returns `None` when
/// the variable is unset.
///
/// Keep the returned guard alive until the benchmarks are done.
pub fn layer<S>() -> Option<(impl Layer<S>, OtelGuard)>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let provider = build_provider()?;
    let layer = tracing_opentelemetry::layer()
        .with_tracer(provider.tracer("codspeed"))
        .with_filter(build_filter());
    Some((layer, OtelGuard { provider }))
}

/// Installs a global `tracing` subscriber exporting spans and events to the receiver at
/// [`ENDPOINT_ENV`], and registers its provider as the global OpenTelemetry tracer provider.
///
/// Returns `None` when the variable is unset or a global subscriber is already set.
pub fn init() -> Option<OtelGuard> {
    if tracing::dispatcher::has_been_set() {
        return None;
    }

    let (layer, guard) = layer()?;
    let subscriber = tracing_subscriber::registry().with(layer);
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        return None;
    }
    opentelemetry::global::set_tracer_provider(guard.provider.clone());
    Some(guard)
}

fn build_provider() -> Option<SdkTracerProvider> {
    let endpoint = std::env::var(ENDPOINT_ENV)
        .ok()
        .filter(|endpoint| !endpoint.is_empty())?;
    let traces_endpoint = format!("{}/v1/traces", endpoint.trim_end_matches('/'));

    let exporter = match SpanExporter::builder()
        .with_http()
        .with_endpoint(traces_endpoint)
        .build()
    {
        Ok(exporter) => exporter,
        Err(error) => {
            eprintln!("Warning: failed to build the OpenTelemetry exporter: {error}");
            return None;
        }
    };

    Some(
        SdkTracerProvider::builder()
            .with_span_processor(BenchmarkLabelProcessor)
            .with_batch_exporter(exporter)
            .with_resource(
                Resource::builder()
                    .with_service_name(service_name())
                    .build(),
            )
            .build(),
    )
}

pub(crate) fn enter_benchmark(uri: &str) {
    *CURRENT_BENCHMARK
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Some(Arc::from(uri));
}

pub(crate) fn exit_benchmark() {
    *CURRENT_BENCHMARK
        .write()
        .unwrap_or_else(PoisonError::into_inner) = None;
}

/// Labels each span with the benchmark running when it starts.
#[derive(Debug)]
struct BenchmarkLabelProcessor;

impl SpanProcessor for BenchmarkLabelProcessor {
    fn on_start(&self, span: &mut Span, _cx: &Context) {
        let current_benchmark = CURRENT_BENCHMARK
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(uri) = current_benchmark {
            span.set_attribute(KeyValue::new(BENCHMARK_URI_ATTRIBUTE, uri));
        }
    }

    fn on_end(&self, _span: SpanData) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

fn build_filter() -> EnvFilter {
    let user_directives = std::env::var(FILTER_ENV).unwrap_or_else(|_| DEFAULT_FILTER.to_owned());
    let directives = std::iter::once(user_directives.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::Tracer as _;
    use std::sync::Mutex;

    #[derive(Debug, Clone, Default)]
    struct RecordingExporter(Arc<Mutex<Vec<SpanData>>>);

    impl opentelemetry_sdk::trace::SpanExporter for RecordingExporter {
        async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
            self.0.lock().unwrap().extend(batch);
            Ok(())
        }
    }

    fn benchmark_uri(span: &SpanData) -> Option<String> {
        span.attributes
            .iter()
            .find(|attribute| attribute.key.as_str() == BENCHMARK_URI_ATTRIBUTE)
            .map(|attribute| attribute.value.to_string())
    }

    #[test]
    fn labels_the_spans_started_in_a_benchmark() {
        let exporter = RecordingExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(BenchmarkLabelProcessor)
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("test");

        tracer.start("before").end();
        enter_benchmark("bench.rs::a");
        tracer.start("inside").end();
        exit_benchmark();
        tracer.start("after").end();

        let labels = exporter
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|span| (span.name.to_string(), benchmark_uri(span)))
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                ("before".into(), None),
                ("inside".into(), Some("bench.rs::a".into())),
                ("after".into(), None),
            ]
        );
    }
}
