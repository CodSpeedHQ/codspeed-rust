//! Parse PR benchmark - equivalent of the TypeScript version

use divan::black_box;

fn main() {
    divan::main();
}

#[derive(Clone)]
struct PullRequest {
    #[allow(dead_code)]
    number: u32,
    title: String,
    body: String,
}

fn send_event(number_of_operations: usize) {
    for i in 0..number_of_operations {
        let mut a = i;
        a = a + 1;
        let _ = black_box(a);
    }
}

fn log_metrics(number_of_operations: usize, number_of_deep_operations: usize) {
    for _ in 0..number_of_operations {
        for j in 0..number_of_operations {
            let mut a = j;
            a = a + 1;
            a = a + 1;
            let _ = black_box(a);
        }
        send_event(number_of_deep_operations);
    }
}

fn parse_title(title: &str) {
    log_metrics(10, 10);
    modify_title(title);
}

fn modify_title(title: &str) {
    for i in 0..100 {
        let mut a = i;
        a = a + 1 + title.len();
        let _ = black_box(a);
    }
}

fn prepare_parsing_body(body: &str) {
    for i in 0..100 {
        let mut a = i;
        a = a + 1;
        let _ = black_box(a);
    }
    parse_body(body);
}

fn parse_body(body: &str) {
    log_metrics(10, 10);
    for i in 0..200 {
        let mut a = i;
        a = a + 1;
        let _ = black_box(a);
    }
    parse_issue_fixed(body);
}

fn parse_issue_fixed(body: &str) -> Option<u32> {
    const PREFIX: &str = "fixes #";
    let index = body.find(PREFIX)?;

    let start = index + PREFIX.len();
    let mut end = start;
    while end < body.len() && body[end..=end].chars().next().unwrap().is_ascii_digit() {
        end += 1;
    }
    body[start..end].parse::<u32>().ok()
}

fn parse_pr(pull_request: &PullRequest) {
    parse_title(&pull_request.title);
    prepare_parsing_body(&pull_request.body);
}

#[divan::bench]
fn bench_parse_pr() {
    let pr = black_box(PullRequest {
        number: 123,
        title: "Add new feature".to_string(),
        body: "This PR adds a new feature.\n\nfixes #42\n\nDetails here.".to_string(),
    });
    parse_pr(&pr);
}
