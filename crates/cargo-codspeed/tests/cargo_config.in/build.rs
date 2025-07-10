fn main() {
    println!("cargo::rustc-check-cfg=cfg(custom_feature_flag)");
}
