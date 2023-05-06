pub fn cli_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("v{}", version);
}
