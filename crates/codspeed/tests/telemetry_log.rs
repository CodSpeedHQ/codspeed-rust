//! Exports the records of the `log` crate through [`telemetry::init`], to an OTLP/HTTP receiver.
//!
//! In a test binary of its own, since `init` installs process-wide loggers.
#![cfg(codspeed_telemetry)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use codspeed::telemetry;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{any_value::Value, KeyValue};
use opentelemetry_proto::tonic::logs::v1::SeverityNumber;
use prost::Message;
use tracing_log::log;

/// Accepts OTLP/HTTP requests on a local port, sending the path and body of each.
fn spawn_receiver() -> (String, mpsc::Receiver<(String, Vec<u8>)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = BufReader::new(stream.unwrap());
            let mut lines = (&mut stream).lines().map(Result::unwrap);
            let path = lines.next().unwrap().split(' ').nth(1).unwrap().to_owned();
            let headers = lines
                .take_while(|header| !header.is_empty())
                .collect::<Vec<_>>();
            // The exporter sends uncompressed bodies of known length.
            let content_length = headers
                .iter()
                .find_map(|header| {
                    let (name, value) = header.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().unwrap())
                })
                .expect("a Content-Length header");
            let mut body = vec![0; content_length];
            stream.read_exact(&mut body).unwrap();
            sender.send((path, body)).unwrap();
            stream
                .get_mut()
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
        }
    });
    (endpoint, receiver)
}

fn attribute(attributes: &[KeyValue], key: &str) -> Option<Value> {
    let attribute = attributes.iter().find(|attribute| attribute.key == key)?;
    attribute.value.clone()?.value
}

fn string(value: &str) -> Option<Value> {
    Some(Value::StringValue(value.to_owned()))
}

#[test]
fn exports_the_records_of_the_log_crate() {
    let (endpoint, requests) = spawn_receiver();
    std::env::set_var("CODSPEED_OTEL_ENDPOINT", endpoint);
    std::env::remove_var("CODSPEED_TELEMETRY_FILTER");
    let guard = telemetry::init().expect("telemetry should be enabled");
    // Taken from the filter of the subscriber: the `log` macros skip the records above it.
    assert_eq!(log::max_level(), log::LevelFilter::Info);

    log::debug!("filtered out by level");
    log::info!(target: "reqwest::connect", "filtered out by target");
    let span = tracing::info_span!("span").entered();
    log::info!("from log");
    let line = line!() - 1;
    drop(span);
    drop(guard);

    let (mut spans, mut records) = (Vec::new(), Vec::new());
    for (path, body) in requests.try_iter() {
        match path.as_str() {
            "/v1/traces" => spans.extend(
                ExportTraceServiceRequest::decode(body.as_slice())
                    .unwrap()
                    .resource_spans
                    .into_iter()
                    .flat_map(|resource| resource.scope_spans)
                    .flat_map(|scope| scope.spans),
            ),
            "/v1/logs" => records.extend(
                ExportLogsServiceRequest::decode(body.as_slice())
                    .unwrap()
                    .resource_logs
                    .into_iter()
                    .flat_map(|resource| resource.scope_logs)
                    .flat_map(|scope| scope.log_records),
            ),
            other => panic!("unexpected request to {other}"),
        }
    }
    let [span] = spans.as_slice() else {
        panic!("expected one span, got {spans:?}");
    };
    let [record] = records.as_slice() else {
        panic!("expected one log record, got {records:?}");
    };
    let body = record.body.clone().and_then(|body| body.value);
    assert_eq!(body, string("from log"));
    assert_eq!(record.severity_number, SeverityNumber::Info as i32);
    assert_eq!(record.severity_text, "INFO");
    assert_eq!(record.event_name, "");
    let attributes = &record.attributes;
    assert_eq!(attribute(attributes, "code.filepath"), string(file!()));
    assert_eq!(
        attribute(attributes, "code.lineno"),
        Some(Value::IntValue(line.into()))
    );
    assert_eq!(
        attribute(attributes, "code.namespace"),
        string(module_path!())
    );
    assert_eq!(record.trace_id, span.trace_id);
    assert_eq!(record.span_id, span.span_id);
}
