fn main() {
    // Check if we're compiling for macOS
    if cfg!(target_os = "macos") {
        // Add the necessary flags for PyO3 on macOS
        println!("cargo:rustc-link-arg=-undefined");
        println!("cargo:rustc-link-arg=dynamic_lookup");
    }
}