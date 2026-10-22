//! Export of the `tracing` spans and events emitted by benchmarks over OTLP/HTTP, as traces and
//! logs respectively.
//!
//! Everything is a no-op unless `CODSPEED_OTEL_ENDPOINT` is set.
//!
//! Ended spans and log records are exported in batches from a background thread. Telemetry is
//! collected in a run of its own, whose performance is not measured, so exporting while a
//! benchmark runs doesn't matter.
//!
//! Spans and log records keep the wall clock times of the SDK. With instrument hooks, they also
//! carry their times on the clock of the benchmark markers: spans as their
//! `codspeed.monotonic_start_ns` and `codspeed.monotonic_end_ns` attributes, log records as their
//! `codspeed.monotonic_time_ns` attribute.

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::logs::{AnyValue, LogRecord as _};
use opentelemetry::trace::{TraceContextExt as _, TracerProvider as _};
use opentelemetry::{InstrumentationScope, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{BatchLogProcessor, LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::trace::{BatchSpanProcessor, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use tracing::dispatcher::WeakDispatch;
use tracing::level_filters::LevelFilter;
use tracing::{span, Dispatch, Subscriber};
use tracing_log::{AsLog, LogTracer};
use tracing_subscriber::filter::{filter_fn, FilterExt};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

use crate::instrument_hooks::InstrumentHooks;

/// Base URL of the OTLP/HTTP receiver, e.g. `http://localhost:4318`.
const ENDPOINT_ENV: &str = "CODSPEED_OTEL_ENDPOINT";

/// `EnvFilter` directives selecting the exported spans and events. Defaults to `info`.
const FILTER_ENV: &str = "CODSPEED_TELEMETRY_FILTER";

const DEFAULT_FILTER: &str = "info";

/// Span attribute holding the time the span started at on the clock of the benchmark markers, in
/// nanoseconds.
const MONOTONIC_START_ATTRIBUTE: &str = "codspeed.monotonic_start_ns";

/// Span attribute holding the time the span ended at on the clock of the benchmark markers, in
/// nanoseconds.
const MONOTONIC_END_ATTRIBUTE: &str = "codspeed.monotonic_end_ns";

/// Log record attribute holding the time the record was emitted at on the clock of the benchmark
/// markers, in nanoseconds.
const MONOTONIC_TIME_ATTRIBUTE: &str = "codspeed.monotonic_time_ns";

/// Ended spans, and log records, queued for export, past which they are dropped. Larger than the
/// SDK default of 2048, which a benchmark emitting on every iteration fills faster than the
/// exporter drains it.
const MAX_QUEUE_SIZE: usize = 1 << 16;

/// Spans or log records sent per export request.
const EXPORT_BATCH_SIZE: usize = 4_096;

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

fn build_layer<S>(providers: &Providers) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    // Without a hint, `filter_fn` would lift the max level of the subscriber to `TRACE`.
    let span_filter =
        filter_fn(|metadata| metadata.is_span()).with_max_level_hint(LevelFilter::TRACE);
    let event_filter =
        filter_fn(|metadata| metadata.is_event()).with_max_level_hint(LevelFilter::TRACE);
    let span_layer = MonotonicEndLayer::default()
        .and_then(tracing_opentelemetry::layer().with_tracer(providers.tracer.tracer("codspeed")))
        .and_then(MonotonicStartLayer::default())
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
            .with_span_processor(span_processor)
            .with_resource(resource.clone())
            .build(),
        logger: SdkLoggerProvider::builder()
            .with_log_processor(GeneratedEventNameProcessor)
            .with_log_processor(MonotonicTimeProcessor)
            .with_log_processor(log_processor)
            .with_resource(resource)
            .build(),
    })
}

/// The time on the clock of the benchmark markers, `None` without instrument hooks.
fn monotonic_now() -> Option<i64> {
    match InstrumentHooks::current_timestamp() {
        0 => None,
        timestamp => i64::try_from(timestamp).ok(),
    }
}

/// Sets an attribute of the OpenTelemetry span of the `tracing` span `id`. Does nothing for the
/// spans the OpenTelemetry layer filters out.
fn set_span_attribute(dispatch: &OnceLock<WeakDispatch>, id: &span::Id, attribute: KeyValue) {
    let Some(dispatch) = dispatch.get().and_then(WeakDispatch::upgrade) else {
        return;
    };
    if let Some(cx) = tracing_opentelemetry::get_otel_context(id, &dispatch) {
        cx.span().set_attribute(attribute);
    }
}

/// Sets the [`MONOTONIC_START_ATTRIBUTE`] of each span when it is created.
///
/// The OpenTelemetry layer only starts the OpenTelemetry span of a `tracing` span when it is first
/// entered, or closed, while it takes its start time when the span is created. This layer starts
/// it right after its creation, to set the attribute then. It must come after the OpenTelemetry
/// layer, whose span it starts.
#[derive(Default)]
struct MonotonicStartLayer {
    dispatch: OnceLock<WeakDispatch>,
}

impl<S> Layer<S> for MonotonicStartLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_register_dispatch(&self, subscriber: &Dispatch) {
        let _ = self.dispatch.set(subscriber.downgrade());
    }

    fn on_new_span(&self, _attrs: &span::Attributes<'_>, id: &span::Id, _ctx: Context<'_, S>) {
        if let Some(timestamp) = monotonic_now() {
            set_span_attribute(
                &self.dispatch,
                id,
                KeyValue::new(MONOTONIC_START_ATTRIBUTE, timestamp),
            );
        }
    }
}

/// The time a span was last exited at, on the clock of the benchmark markers.
struct LastExit(i64);

/// Sets the [`MONOTONIC_END_ATTRIBUTE`] of each span when it is closed: the time it was last
/// exited at, or the time it is closed at when it was never entered, as the OpenTelemetry layer
/// takes its end time.
///
/// The OpenTelemetry layer ends the span when it is closed, and the SDK only adds attributes, so
/// the attribute is set once, then. This layer must come before the OpenTelemetry layer.
#[derive(Default)]
struct MonotonicEndLayer {
    dispatch: OnceLock<WeakDispatch>,
}

impl<S> Layer<S> for MonotonicEndLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_register_dispatch(&self, subscriber: &Dispatch) {
        let _ = self.dispatch.set(subscriber.downgrade());
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let (Some(timestamp), Some(span)) = (monotonic_now(), ctx.span(id)) {
            // A span entered on several threads is exited concurrently, and the exits may take
            // the lock in another order than they read the clock in.
            let mut extensions = span.extensions_mut();
            match extensions.get_mut::<LastExit>() {
                Some(last_exit) => last_exit.0 = last_exit.0.max(timestamp),
                None => extensions.insert(LastExit(timestamp)),
            }
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let last_exit = ctx
            .span(&id)
            .and_then(|span| span.extensions().get::<LastExit>().map(|exit| exit.0));
        if let Some(timestamp) = last_exit.or_else(monotonic_now) {
            set_span_attribute(
                &self.dispatch,
                &id,
                KeyValue::new(MONOTONIC_END_ATTRIBUTE, timestamp),
            );
        }
    }
}

/// Sets the [`MONOTONIC_TIME_ATTRIBUTE`] of the emitted log records, which are emitted as their
/// event happens.
#[derive(Debug)]
struct MonotonicTimeProcessor;

impl LogProcessor for MonotonicTimeProcessor {
    fn emit(&self, record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        if let Some(timestamp) = monotonic_now() {
            record.add_attribute(MONOTONIC_TIME_ATTRIBUTE, timestamp);
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// Event name `tracing-log` gives every record of the `log` crate.
const LOG_EVENT_NAME: &str = "log event";

/// Clears the event names `tracing` generates for the events given none, so that only the
/// events named at their callsite are exported with a name. Their location stays in the
/// `code.*` attributes.
#[derive(Debug)]
struct GeneratedEventNameProcessor;

impl LogProcessor for GeneratedEventNameProcessor {
    fn emit(&self, record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        if has_generated_event_name(record) {
            record.set_event_name("");
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// Whether the event name is `log event`, or the `event <file>:<line>` that `tracing` generates
/// from the location of the record.
fn has_generated_event_name(record: &SdkLogRecord) -> bool {
    let Some(name) = record.event_name() else {
        return false;
    };
    if name == LOG_EVENT_NAME {
        return true;
    }
    let attribute = |key: &str| {
        record
            .attributes_iter()
            .find(|(attribute_key, _)| attribute_key.as_str() == key)
            .map(|(_, value)| value)
    };
    let (Some(AnyValue::String(file)), Some(&AnyValue::Int(line))) =
        (attribute("code.filepath"), attribute("code.lineno"))
    else {
        return false;
    };
    name.strip_prefix("event ")
        .and_then(|location| location.strip_prefix(file.as_str()))
        .and_then(|location| location.strip_prefix(':'))
        .is_some_and(|location| location.parse() == Ok(line))
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
    use opentelemetry::Value;
    use opentelemetry_sdk::logs::LogBatch;
    use opentelemetry_sdk::trace::SpanData;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, Default)]
    struct RecordingExporter(Arc<Mutex<Vec<SpanData>>>);

    impl opentelemetry_sdk::trace::SpanExporter for RecordingExporter {
        async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
            self.0.lock().unwrap().extend(batch);
            Ok(())
        }
    }

    #[derive(Debug, Clone, Default)]
    struct RecordingLogExporter(Arc<Mutex<Vec<SdkLogRecord>>>);

    impl opentelemetry_sdk::logs::LogExporter for RecordingLogExporter {
        async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
            let records = batch.iter().map(|(record, _)| record.clone());
            self.0.lock().unwrap().extend(records);
            Ok(())
        }
    }

    fn log_body(record: &SdkLogRecord) -> String {
        match record.body() {
            Some(AnyValue::String(body)) => body.to_string(),
            other => panic!("unexpected log body {other:?}"),
        }
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

    #[test]
    fn clears_the_event_names_generated_by_tracing() {
        let log_exporter = RecordingLogExporter::default();
        let providers = Providers {
            tracer: SdkTracerProvider::builder().build(),
            logger: SdkLoggerProvider::builder()
                .with_log_processor(GeneratedEventNameProcessor)
                .with_simple_exporter(log_exporter.clone())
                .build(),
        };
        let subscriber = tracing_subscriber::registry().with(build_layer(&providers));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("unnamed");
            tracing::info!(name: "cache.miss", "named");
        });

        let records = log_exporter.0.lock().unwrap();
        let names = records
            .iter()
            .map(|record| (log_body(record), record.event_name()))
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                ("unnamed".to_owned(), Some("")),
                ("named".to_owned(), Some("cache.miss")),
            ]
        );
        let has_file = records[0].attributes_iter().any(|(key, value)| {
            key.as_str() == "code.filepath" && value == &AnyValue::from(file!())
        });
        assert!(has_file);
    }

    fn providers(
        span_exporter: &RecordingExporter,
        log_exporter: &RecordingLogExporter,
    ) -> Providers {
        Providers {
            tracer: SdkTracerProvider::builder()
                .with_simple_exporter(span_exporter.clone())
                .build(),
            logger: SdkLoggerProvider::builder()
                .with_log_processor(MonotonicTimeProcessor)
                .with_simple_exporter(log_exporter.clone())
                .build(),
        }
    }

    fn span_attribute(span: &SpanData, key: &str) -> i64 {
        let value = span
            .attributes
            .iter()
            .find(|attribute| attribute.key.as_str() == key)
            .map(|attribute| attribute.value.clone());
        match value {
            Some(Value::I64(value)) => value,
            _ => panic!("no {key} in {:?}", span.attributes),
        }
    }

    #[cfg(all(use_instrument_hooks, any(target_os = "linux", target_os = "macos")))]
    #[test]
    fn records_the_monotonic_time_of_spans_and_log_records() {
        let span_exporter = RecordingExporter::default();
        let log_exporter = RecordingLogExporter::default();
        let subscriber = tracing_subscriber::registry()
            .with(build_layer(&providers(&span_exporter, &log_exporter)));
        let now = || InstrumentHooks::current_timestamp() as i64;
        let pause = || std::thread::sleep(Duration::from_millis(1));

        let times = tracing::subscriber::with_default(subscriber, || {
            let before = now();
            let span = tracing::info_span!("entered after its creation");
            let created = now();
            pause();
            let entered = span.enter();
            tracing::info!("inside");
            let logged = now();
            drop(entered);
            let exited = now();
            pause();
            drop(span);
            let never_entered = tracing::info_span!("never entered");
            pause();
            let closing = now();
            drop(never_entered);
            let closed = now();
            [before, created, logged, exited, closing, closed]
        });
        let [before, created, logged, exited, closing, closed] = times;

        let spans = span_exporter.0.lock().unwrap();
        let [span, never_entered] = &spans[..] else {
            panic!("expected two spans, got {spans:?}");
        };
        // Started when created, not when entered, and ended when last exited, not when closed.
        let start = span_attribute(span, MONOTONIC_START_ATTRIBUTE);
        assert!(before <= start && start <= created);
        let end = span_attribute(span, MONOTONIC_END_ATTRIBUTE);
        assert!(logged <= end && end <= exited);
        // Ended when closed.
        let end = span_attribute(never_entered, MONOTONIC_END_ATTRIBUTE);
        assert!(closing <= end && end <= closed);

        let records = log_exporter.0.lock().unwrap();
        let time = records[0]
            .attributes_iter()
            .find(|(key, _)| key.as_str() == MONOTONIC_TIME_ATTRIBUTE)
            .map(|(_, value)| value.clone());
        let Some(AnyValue::Int(time)) = time else {
            panic!("no monotonic time in {:?}", records[0]);
        };
        assert!(created <= time && time <= logged);
    }
}
