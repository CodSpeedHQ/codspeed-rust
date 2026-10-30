//! Tests of the export, through the layer [`init`](super::init) installs.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opentelemetry::logs::AnyValue;
use opentelemetry::Value;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanData};
use tracing_subscriber::layer::SubscriberExt;

use super::export::{build_layer, Providers};
use super::monotonic::{
    MonotonicTimeProcessor, MONOTONIC_END_ATTRIBUTE, MONOTONIC_START_ATTRIBUTE,
    MONOTONIC_TIME_ATTRIBUTE,
};

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

fn providers(span_exporter: &RecordingExporter, log_exporter: &RecordingLogExporter) -> Providers {
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
    let subscriber =
        tracing_subscriber::registry().with(build_layer(&providers(&span_exporter, &log_exporter)));
    let now = || crate::instrument_hooks::InstrumentHooks::current_timestamp() as i64;
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
