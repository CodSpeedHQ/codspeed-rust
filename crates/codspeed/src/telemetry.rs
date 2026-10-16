//! Export of the `tracing` spans and events emitted by benchmarks over OTLP/HTTP, as traces and
//! logs respectively.
//!
//! Everything is a no-op unless `CODSPEED_OTEL_ENDPOINT` is set.

use std::time::Duration;

use opentelemetry::logs::{AnyValue, LogRecord as _};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::filter::{filter_fn, FilterExt};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, EnvFilter, Layer};

/// Base URL of the OTLP/HTTP receiver, e.g. `http://localhost:4318`.
const ENDPOINT_ENV: &str = "CODSPEED_OTEL_ENDPOINT";

/// `EnvFilter` directives selecting the exported spans and events. Defaults to `info`.
const FILTER_ENV: &str = "CODSPEED_TELEMETRY_FILTER";

const DEFAULT_FILTER: &str = "info";

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
    let span_layer = tracing_opentelemetry::layer()
        .with_tracer(providers.tracer.tracer("codspeed"))
        .with_filter(build_filter().and(filter_fn(|metadata| metadata.is_span())));
    let log_layer = OpenTelemetryTracingBridge::new(&providers.logger)
        .with_filter(build_filter().and(filter_fn(|metadata| metadata.is_event())));
    span_layer.and_then(log_layer)
}

/// Installs a global `tracing` subscriber exporting spans and events to the receiver at
/// `CODSPEED_OTEL_ENDPOINT`, and registers its provider as the global OpenTelemetry tracer
/// provider.
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
            .with_batch_exporter(span_exporter)
            .with_resource(resource.clone())
            .build(),
        logger: SdkLoggerProvider::builder()
            .with_log_processor(GeneratedEventNameProcessor)
            .with_batch_exporter(log_exporter)
            .with_resource(resource)
            .build(),
    })
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
    use opentelemetry::logs::AnyValue;
    use opentelemetry_sdk::error::OTelSdkResult;
    use opentelemetry_sdk::logs::{LogBatch, SdkLogRecord};
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
}
