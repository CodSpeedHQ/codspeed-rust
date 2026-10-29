//! The warm-up of the subscriber, and the filtering of what it emits out of the export.

use std::time::Duration;

use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord};
use opentelemetry_sdk::trace::{Span, SpanData, SpanProcessor};
use opentelemetry_sdk::Resource;

/// Target of the span and event [`warm_up`] emits.
const WARMUP_TARGET: &str = "codspeed::telemetry::warmup";

/// Field marking the span and event [`warm_up`] emits, which are not exported.
const WARMUP_FIELD: &str = "codspeed_warmup";

/// The first span and the first event of the process take tens of microseconds longer than the
/// next ones to go through the subscriber. Emits one of each, which are not exported, so that the
/// first benchmark emitting telemetry doesn't take that time.
pub(super) fn warm_up() {
    let span = tracing::error_span!(target: WARMUP_TARGET, "warmup", codspeed_warmup = true);
    let _entered = span.enter();
    tracing::error!(target: WARMUP_TARGET, codspeed_warmup = true, "warmup");
}

/// The filter directive letting the span and event of [`warm_up`] through, whatever the filter
/// of the user. At `error`, their level, so that it doesn't raise the max level of the subscriber.
pub(super) fn filter_directive() -> String {
    format!("{WARMUP_TARGET}=error")
}

/// Drops the span and event [`warm_up`] emits, before handing the others to the processor it
/// wraps.
#[derive(Debug)]
pub(super) struct WarmupFilter<P>(pub(super) P);

impl<P: SpanProcessor> SpanProcessor for WarmupFilter<P> {
    fn on_start(&self, span: &mut Span, cx: &opentelemetry::Context) {
        self.0.on_start(span, cx);
    }

    fn on_end(&self, span: SpanData) {
        let is_warmup = span
            .attributes
            .iter()
            .any(|attribute| attribute.key.as_str() == WARMUP_FIELD);
        if !is_warmup {
            self.0.on_end(span);
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.0.set_resource(resource);
    }
}

impl<P: LogProcessor> LogProcessor for WarmupFilter<P> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        let is_warmup = record
            .attributes_iter()
            .any(|(key, _)| key.as_str() == WARMUP_FIELD);
        if !is_warmup {
            self.0.emit(record, instrumentation);
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.0.set_resource(resource);
    }
}
