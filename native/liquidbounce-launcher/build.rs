fn main() {
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/launcher.ico")
        .set("ProductName", "LiquidBounce - Rust")
        .set("FileDescription", "LiquidBounce Rust 26.3 Launcher")
        .compile()
        .expect("compile launcher icon resource");
}
