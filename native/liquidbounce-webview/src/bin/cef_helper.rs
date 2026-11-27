#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    let result = liquidbounce_webview::runtime::execute_subprocess();
    std::process::exit(if result >= 0 { result } else { 2 });
}
