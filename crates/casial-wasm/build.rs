fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        println!("cargo:rustc-cfg=getrandom_backend=\"wasm_js\"");
    }
}
