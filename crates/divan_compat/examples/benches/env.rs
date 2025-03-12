fn main() {
    divan::main();
}

#[divan::bench]
fn print_env_hello() {
    let env_var = std::env::var("MY_ENV_VAR").unwrap_or("not set".to_string());
    println!("MY_ENV_VAR is {}", env_var);
}
