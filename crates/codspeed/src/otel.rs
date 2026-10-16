//! Export of the `tracing` spans and events emitted by benchmarks over OTLP/HTTP, as traces and
//! logs respectively.
//!
//! Everything is a no-op unless [`ENDPOINT_ENV`] is set.
//!
//! Ended spans and log records are held in memory and only exported when a benchmark scope exits
//! (see [`crate::benchmark_scope`]) or the exporters shut down, so no export runs while a benchmark
//! is measured.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock};
use std::time::Duration;

use opentelemetry::logs::{AnyValue, LogRecord as _};
use opentelemetry::trace::{Span as _, TracerProvider as _};
use opentelemetry::{Context, InstrumentationScope, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::trace::{SdkTracerProvider, Span, SpanData, SpanProcessor};
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::filter::{filter_fn, FilterExt};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, EnvFilter, Layer};

/// Base URL of the OTLP/HTTP receiver, e.g. `http://localhost:4318`.
pub const ENDPOINT_ENV: &str = "CODSPEED_OTEL_ENDPOINT";

/// `EnvFilter` directives selecting the exported spans and events. Defaults to `info`.
pub const FILTER_ENV: &str = "CODSPEED_OTEL_FILTER";

const DEFAULT_FILTER: &str = "info";

/// Span and log record attribute holding the URI of the benchmark running when the span started
/// or the record was emitted.
pub const BENCHMARK_URI_ATTRIBUTE: &str = "codspeed.benchmark.uri";

/// Ended spans, and log records, held in memory between two exports, past which they are dropped.
/// Bounds the memory used by a benchmark emitting on every iteration.
const MAX_HELD: usize = 1_000_000;

/// Spans or log records sent per export request.
const EXPORT_BATCH_SIZE: usize = 4_096;

static CURRENT_BENCHMARK: RwLock<Option<Arc<str>>> = RwLock::new(None);

static PROVIDERS: OnceLock<Providers> = OnceLock::new();

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

#[derive(Debug, Clone)]
struct Providers {
    tracer: SdkTracerProvider,
    logger: SdkLoggerProvider,
}

impl Providers {
    fn flush(&self) {
        if let Err(error) = self.tracer.force_flush() {
            eprintln!("Warning: failed to export OpenTelemetry spans: {error}");
        }
        if let Err(error) = self.logger.force_flush() {
            eprintln!("Warning: failed to export OpenTelemetry logs: {error}");
        }
    }
}

/// Flushes and shuts down the exporters when dropped.
#[must_use = "dropping the guard shuts the exporters down"]
pub struct OtelGuard {
    providers: Providers,
}

impl Drop for OtelGuard {
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
/// [`ENDPOINT_ENV`], to add to a subscriber of your own. Returns `None` when
/// the variable is unset.
///
/// Events are exported as log records carrying the trace and span IDs of the span they are emitted
/// in, rather than as span events.
///
/// Keep the returned guard alive until the benchmarks are done.
pub fn layer<S>() -> Option<(impl Layer<S>, OtelGuard)>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let providers = build_providers()?;
    let _ = PROVIDERS.set(providers.clone());
    Some((build_layer(&providers), OtelGuard { providers }))
}

fn build_layer<S>(providers: &Providers) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let span_layer = tracing_opentelemetry::layer()
        .with_tracer(providers.tracer.tracer("codspeed"))
        .with_filter(build_filter().and(filter_fn(|metadata| metadata.is_span())));
    let log_layer = OpenTelemetryTracingBridge::new(&providers.logger)
        .with_filter(build_filter().and(filter_fn(|metadata| metadata.is_event())));
    span_layer.and_then(log_layer)
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
    opentelemetry::global::set_tracer_provider(guard.providers.tracer.clone());
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
    Some(Providers {
        tracer: SdkTracerProvider::builder()
            .with_span_processor(BenchmarkLabelProcessor)
            .with_span_processor(HoldingSpanProcessor::new(span_exporter))
            .with_resource(resource.clone())
            .build(),
        logger: SdkLoggerProvider::builder()
            .with_log_processor(BenchmarkLabelProcessor)
            .with_log_processor(HoldingLogProcessor::new(log_exporter))
            .with_resource(resource)
            .build(),
    })
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

    if let Some(providers) = PROVIDERS.get() {
        providers.flush();
    }
}

fn current_benchmark() -> Option<Arc<str>> {
    CURRENT_BENCHMARK
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Labels each span with the benchmark running when it starts, and each log record with the one
/// running when it is emitted.
#[derive(Debug)]
struct BenchmarkLabelProcessor;

impl SpanProcessor for BenchmarkLabelProcessor {
    fn on_start(&self, span: &mut Span, _cx: &Context) {
        if let Some(uri) = current_benchmark() {
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

impl LogProcessor for BenchmarkLabelProcessor {
    fn emit(&self, record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        if let Some(uri) = current_benchmark() {
            record.add_attribute(BENCHMARK_URI_ATTRIBUTE, AnyValue::String(uri.into()));
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// Items held until a flush, up to [`MAX_HELD`].
#[derive(Debug)]
struct Held<T> {
    items: Mutex<Vec<T>>,
    dropped: AtomicUsize,
}

impl<T> Held<T> {
    fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            dropped: AtomicUsize::new(0),
        }
    }

    fn push(&self, item: T) {
        let mut items = self.items.lock().unwrap_or_else(PoisonError::into_inner);
        if items.len() < MAX_HELD {
            items.push(item);
        } else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Takes the held items, warning about those dropped since the last take.
    fn take(&self, kind: &str) -> Vec<T> {
        let items = std::mem::take(&mut *self.items.lock().unwrap_or_else(PoisonError::into_inner));
        let dropped = self.dropped.swap(0, Ordering::Relaxed);
        if dropped > 0 {
            eprintln!(
                "Warning: dropped {dropped} OpenTelemetry {kind}, past the {MAX_HELD} held per \
                 benchmark"
            );
        }
        items
    }
}

/// Exports in batches, carrying on past a failed batch.
fn export_in_batches<T>(
    items: Vec<T>,
    mut export: impl FnMut(Vec<T>) -> OTelSdkResult,
) -> OTelSdkResult {
    let mut result = Ok(());
    let mut items = items.into_iter().peekable();
    while items.peek().is_some() {
        if let Err(error) = export(items.by_ref().take(EXPORT_BATCH_SIZE).collect()) {
            result = Err(error);
        }
    }
    result
}

/// Holds the ended spans until a flush.
#[derive(Debug)]
struct HoldingSpanProcessor<E> {
    exporter: E,
    spans: Held<SpanData>,
}

impl<E> HoldingSpanProcessor<E> {
    fn new(exporter: E) -> Self {
        Self {
            exporter,
            spans: Held::new(),
        }
    }
}

impl<E: opentelemetry_sdk::trace::SpanExporter> SpanProcessor for HoldingSpanProcessor<E> {
    fn on_start(&self, _span: &mut Span, _cx: &Context) {}

    fn on_end(&self, span: SpanData) {
        self.spans.push(span);
    }

    fn force_flush(&self) -> OTelSdkResult {
        export_in_batches(self.spans.take("spans"), |batch| {
            futures_executor::block_on(self.exporter.export(batch))
        })
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.force_flush()
            .and(self.exporter.shutdown_with_timeout(timeout))
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.exporter.set_resource(resource);
    }
}

/// Holds the emitted log records until a flush.
#[derive(Debug)]
struct HoldingLogProcessor<E> {
    exporter: E,
    records: Held<(SdkLogRecord, InstrumentationScope)>,
}

impl<E> HoldingLogProcessor<E> {
    fn new(exporter: E) -> Self {
        Self {
            exporter,
            records: Held::new(),
        }
    }
}

impl<E: opentelemetry_sdk::logs::LogExporter> LogProcessor for HoldingLogProcessor<E> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        self.records.push((record.clone(), instrumentation.clone()));
    }

    fn force_flush(&self) -> OTelSdkResult {
        export_in_batches(self.records.take("log records"), |batch| {
            let batch = batch
                .iter()
                .map(|(record, instrumentation)| (record, instrumentation))
                .collect::<Vec<_>>();
            futures_executor::block_on(self.exporter.export(LogBatch::new(&batch)))
        })
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.force_flush()
            .and(self.exporter.shutdown_with_timeout(timeout))
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.exporter.set_resource(resource);
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
    use opentelemetry::logs::{Logger as _, LoggerProvider as _};
    use opentelemetry::trace::Tracer as _;

    /// Records the exported spans, and the size of each batch.
    #[derive(Debug, Clone, Default)]
    struct RecordingExporter(Arc<Mutex<Vec<SpanData>>>, Arc<Mutex<Vec<usize>>>);

    impl opentelemetry_sdk::trace::SpanExporter for RecordingExporter {
        async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
            self.1.lock().unwrap().push(batch.len());
            self.0.lock().unwrap().extend(batch);
            Ok(())
        }
    }

    /// Records the exported log records, and the size of each batch.
    #[derive(Debug, Clone, Default)]
    struct RecordingLogExporter(Arc<Mutex<Vec<SdkLogRecord>>>, Arc<Mutex<Vec<usize>>>);

    impl opentelemetry_sdk::logs::LogExporter for RecordingLogExporter {
        async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
            let records = batch.iter().map(|(record, _)| record.clone());
            let mut recorded = self.0.lock().unwrap();
            let len_before = recorded.len();
            recorded.extend(records);
            self.1.lock().unwrap().push(recorded.len() - len_before);
            Ok(())
        }
    }

    fn benchmark_uri(span: &SpanData) -> Option<String> {
        span.attributes
            .iter()
            .find(|attribute| attribute.key.as_str() == BENCHMARK_URI_ATTRIBUTE)
            .map(|attribute| attribute.value.to_string())
    }

    fn log_benchmark_uri(record: &SdkLogRecord) -> Option<String> {
        record
            .attributes_iter()
            .find(|(key, _)| key.as_str() == BENCHMARK_URI_ATTRIBUTE)
            .map(|(_, value)| match value {
                AnyValue::String(uri) => uri.to_string(),
                other => panic!("unexpected benchmark URI {other:?}"),
            })
    }

    fn log_body(record: &SdkLogRecord) -> String {
        match record.body() {
            Some(AnyValue::String(body)) => body.to_string(),
            other => panic!("unexpected log body {other:?}"),
        }
    }

    // Spans and log records are checked in one test since the current benchmark is global.
    #[test]
    fn labels_the_spans_and_log_records_of_a_benchmark() {
        let span_exporter = RecordingExporter::default();
        let tracer_provider = SdkTracerProvider::builder()
            .with_span_processor(BenchmarkLabelProcessor)
            .with_simple_exporter(span_exporter.clone())
            .build();
        let tracer = tracer_provider.tracer("test");
        let log_exporter = RecordingLogExporter::default();
        let logger_provider = SdkLoggerProvider::builder()
            .with_log_processor(BenchmarkLabelProcessor)
            .with_simple_exporter(log_exporter.clone())
            .build();
        let logger = logger_provider.logger("test");
        let log = |body: &'static str| {
            let mut record = logger.create_log_record();
            record.set_body(body.into());
            logger.emit(record);
        };

        tracer.start("before").end();
        log("before");
        enter_benchmark("bench.rs::a");
        tracer.start("inside").end();
        log("inside");
        exit_benchmark();
        tracer.start("after").end();
        log("after");

        let expected: [(String, Option<String>); 3] = [
            ("before".into(), None),
            ("inside".into(), Some("bench.rs::a".into())),
            ("after".into(), None),
        ];
        let span_labels = span_exporter
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|span| (span.name.to_string(), benchmark_uri(span)))
            .collect::<Vec<_>>();
        assert_eq!(span_labels, expected);
        let log_labels = log_exporter
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|record| (log_body(record), log_benchmark_uri(record)))
            .collect::<Vec<_>>();
        assert_eq!(log_labels, expected);
    }

    #[test]
    fn holds_spans_until_flushed() {
        let exporter = RecordingExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(HoldingSpanProcessor::new(exporter.clone()))
            .build();
        let tracer = provider.tracer("test");

        for _ in 0..=EXPORT_BATCH_SIZE {
            tracer.start("span").end();
        }
        assert!(exporter.0.lock().unwrap().is_empty());

        provider.force_flush().unwrap();
        assert_eq!(exporter.0.lock().unwrap().len(), EXPORT_BATCH_SIZE + 1);
        assert_eq!(*exporter.1.lock().unwrap(), [EXPORT_BATCH_SIZE, 1]);
    }

    #[test]
    fn holds_log_records_until_flushed() {
        let exporter = RecordingLogExporter::default();
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(HoldingLogProcessor::new(exporter.clone()))
            .build();
        let logger = provider.logger("test");

        for _ in 0..=EXPORT_BATCH_SIZE {
            logger.emit(logger.create_log_record());
        }
        assert!(exporter.0.lock().unwrap().is_empty());

        provider.force_flush().unwrap();
        assert_eq!(exporter.0.lock().unwrap().len(), EXPORT_BATCH_SIZE + 1);
        assert_eq!(*exporter.1.lock().unwrap(), [EXPORT_BATCH_SIZE, 1]);
    }

    #[test]
    fn exports_events_as_log_records_of_their_span() {
        let span_exporter = RecordingExporter::default();
        let log_exporter = RecordingLogExporter::default();
        let providers = Providers {
            tracer: SdkTracerProvider::builder()
                .with_simple_exporter(span_exporter.clone())
                .build(),
            logger: SdkLoggerProvider::builder()
                .with_simple_exporter(log_exporter.clone())
                .build(),
        };
        let subscriber = tracing_subscriber::registry().with(build_layer(&providers));

        tracing::subscriber::with_default(subscriber, || {
            let _span = tracing::info_span!("parent").entered();
            tracing::info!("inside");
        });

        let spans = span_exporter.0.lock().unwrap();
        let [span] = spans.as_slice() else {
            panic!("expected one span, got {spans:?}");
        };
        assert!(span.events.is_empty());
        let records = log_exporter.0.lock().unwrap();
        let [record] = records.as_slice() else {
            panic!("expected one log record, got {records:?}");
        };
        assert_eq!(log_body(record), "inside");
        let trace_context = record.trace_context().unwrap();
        assert_eq!(trace_context.trace_id, span.span_context.trace_id());
        assert_eq!(trace_context.span_id, span.span_context.span_id());
    }
}
