//! The times of the spans and log records on the clock of the benchmark markers.

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::logs::LogRecord as _;
use opentelemetry::trace::TraceContextExt as _;
use opentelemetry::{InstrumentationScope, KeyValue};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord};
use tracing::dispatcher::WeakDispatch;
use tracing::{span, Dispatch, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::instrument_hooks::InstrumentHooks;

/// Span attribute holding the time the span started at on the clock of the benchmark markers, in
/// nanoseconds.
pub(super) const MONOTONIC_START_ATTRIBUTE: &str = "codspeed.monotonic_start_ns";

/// Span attribute holding the time the span ended at on the clock of the benchmark markers, in
/// nanoseconds.
pub(super) const MONOTONIC_END_ATTRIBUTE: &str = "codspeed.monotonic_end_ns";

/// Log record attribute holding the time the record was emitted at on the clock of the benchmark
/// markers, in nanoseconds.
pub(super) const MONOTONIC_TIME_ATTRIBUTE: &str = "codspeed.monotonic_time_ns";

/// The time a span was created at, on the clock of the benchmark markers.
struct Start(i64);

/// The time a span was last exited at, on the clock of the benchmark markers.
struct LastExit(i64);

/// Records when each span starts and ends on the clock of the benchmark markers, as its
/// [`MONOTONIC_START_ATTRIBUTE`] and [`MONOTONIC_END_ATTRIBUTE`]:
/// - the start is when the span is created;
/// - the end is when the span is last exited, or when it is closed if it was never entered.
///
/// These are the moments the OpenTelemetry layer takes for the span's own start and end times.
///
/// Both attributes are set when the span is closed. This layer must come before the
/// OpenTelemetry layer: it then reads the start time before the OpenTelemetry layer processes the
/// new span, and sets the attributes before the OpenTelemetry layer ends and exports the span.
#[derive(Default)]
pub(super) struct MonotonicTimestampLayer {
    dispatch: OnceLock<WeakDispatch>,
}

impl<S> Layer<S> for MonotonicTimestampLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_register_dispatch(&self, subscriber: &Dispatch) {
        let _ = self.dispatch.set(subscriber.downgrade());
    }

    fn on_new_span(&self, _attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut()
                .insert(Start(InstrumentHooks::current_timestamp() as i64));
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            // A span entered on several threads is exited concurrently, and the exits may take
            // the lock in another order than they read the clock in.
            let timestamp = InstrumentHooks::current_timestamp() as i64;
            let mut extensions = span.extensions_mut();
            match extensions.get_mut::<LastExit>() {
                Some(last_exit) => last_exit.0 = last_exit.0.max(timestamp),
                None => extensions.insert(LastExit(timestamp)),
            }
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else {
            return;
        };
        // Release the lock on the extensions before calling `get_otel_context`, which locks them
        // too.
        let (start, last_exit) = {
            let extensions = span.extensions();
            (
                extensions.get::<Start>().map(|start| start.0),
                extensions.get::<LastExit>().map(|exit| exit.0),
            )
        };
        let attributes = [
            start.map(|start| KeyValue::new(MONOTONIC_START_ATTRIBUTE, start)),
            last_exit
                .or_else(|| Some(InstrumentHooks::current_timestamp() as i64))
                .map(|end| KeyValue::new(MONOTONIC_END_ATTRIBUTE, end)),
        ];

        // Nothing to set for the spans the OpenTelemetry layer filters out.
        let Some(dispatch) = self.dispatch.get().and_then(WeakDispatch::upgrade) else {
            return;
        };
        if let Some(cx) = tracing_opentelemetry::get_otel_context(&id, &dispatch) {
            cx.span().set_attributes(attributes.into_iter().flatten());
        }
    }
}

/// Sets the [`MONOTONIC_TIME_ATTRIBUTE`] of the emitted log records, which are emitted as their
/// event happens.
#[derive(Debug)]
pub(super) struct MonotonicTimeProcessor;

impl LogProcessor for MonotonicTimeProcessor {
    fn emit(&self, record: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        record.add_attribute(
            MONOTONIC_TIME_ATTRIBUTE,
            InstrumentHooks::current_timestamp() as i64,
        );
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}
