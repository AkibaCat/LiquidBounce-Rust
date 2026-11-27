#[path = "cef_smoke/lifecycle.rs"]
mod lifecycle;

use liquidbounce_webview::{
    input::InputQueue,
    paint::Mailbox,
    runtime::{self, Config},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn capture(
    mailbox: &Mailbox,
    output: &Path,
    settle: Duration,
    fixture: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(30) + settle;
    let mut loaded_since = None;
    loop {
        runtime::pump_current_thread()?;
        let page = runtime::page_state_current_thread().ok_or("CEF runtime disappeared")?;
        if !page.load_errors.is_empty() {
            return Err(format!("CEF main document load failed: {:?}", page.load_errors).into());
        }
        if page.http_status.is_some_and(|status| status >= 400) {
            return Err(format!("CEF main document HTTP status {:?}", page.http_status).into());
        }
        if page.completed_loads > 0 && !page.loading && mailbox.paint_count() > 0 {
            if loaded_since.get_or_insert_with(Instant::now).elapsed() >= settle {
                break;
            }
        } else {
            loaded_since = None;
        }
        if Instant::now() >= deadline {
            return Err(format!("CEF document/paint timeout: {page:?}").into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let image = mailbox
        .snapshot()
        .view
        .ok_or("CEF paint image is missing")?;
    let mut rgba = Vec::with_capacity(image.bgra.len());
    for bgra in image.bgra.chunks_exact(4) {
        let alpha = u16::from(bgra[3]);
        let channel = |value: u8| {
            if alpha == 0 {
                0
            } else {
                ((u16::from(value) * 255 + alpha / 2) / alpha).min(255) as u8
            }
        };
        rgba.extend_from_slice(&[
            channel(bgra[2]),
            channel(bgra[1]),
            channel(bgra[0]),
            bgra[3],
        ]);
    }
    let sample = |x: u32, y: u32| -> [u8; 4] {
        let offset = (y * image.width + x) as usize * 4;
        rgba[offset..offset + 4].try_into().unwrap()
    };
    let samples = [sample(100, 200), sample(500, 200), sample(700, 200)];
    if fixture && samples != [[255, 0, 0, 128], [0, 255, 0, 255], [0, 0, 0, 0]] {
        return Err(format!("CEF fixture pixels differ: {samples:?}").into());
    }
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create(output)?),
        image.width,
        image.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba)?;
    writer.finish()?;
    Ok(serde_json::json!({
        "width": image.width, "height": image.height,
        "paintCount": mailbox.paint_count(), "sampleRgba": samples, "png": output,
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let supplied_url = args.next().unwrap_or_else(|| "--fixture".into());
    let lifecycle_mode = supplied_url == "--lifecycle";
    let fixture = supplied_url == "--fixture";
    let url = if lifecycle_mode {
        lifecycle::initial_url()
    } else if fixture {
        let html = include_str!("../../fixtures/osr-smoke.html");
        let encoded: String = html
            .as_bytes()
            .iter()
            .map(|b| format!("%{b:02X}"))
            .collect();
        format!("data:text/html;charset=utf-8,{encoded}")
    } else {
        supplied_url
    };
    let output = PathBuf::from(args.next().unwrap_or_else(|| "cef-osr-smoke.png".into()));
    let settle = Duration::from_secs(
        args.next()
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or(2),
    );
    let report = PathBuf::from(
        args.next()
            .unwrap_or_else(|| output.with_extension("json").to_string_lossy().into_owned()),
    );
    let resources = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let cache = std::env::temp_dir().join(format!(
        "liquidbounce-cef-smoke-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    ));
    let mailbox = Arc::new(Mailbox::new([800, 450]));
    let input = InputQueue::default();
    runtime::initialize_current_thread(
        Config {
            url,
            click_gui_url: None,
            helper_executable: resources.join("liquidbounce-cef-helper.exe"),
            cache_directory: cache.clone(),
            resources_directory: resources,
        },
        mailbox.clone(),
        input.clone(),
    )?;
    let mut lifecycle_evidence = Vec::new();
    let mut result = if lifecycle_mode {
        lifecycle::run(&mailbox, &input, &mut lifecycle_evidence)
            .and_then(|()| capture(&mailbox, &output, settle, false))
    } else {
        capture(&mailbox, &output, settle, fixture)
    };
    let page = runtime::page_state_current_thread().unwrap_or_default();
    if lifecycle_mode && result.is_ok() {
        if let Err(error) = lifecycle::prepare_shutdown(&mailbox, &input, &mut lifecycle_evidence) {
            result = Err(error);
        }
    }
    let shutdown_start = runtime::page_status_current_thread();
    // Always close on the initializing thread, including timeout, HTTP and image write errors.
    let shutdown = runtime::shutdown_current_thread(Duration::from_secs(5));
    let runtime_released = runtime::page_status_current_thread().is_none();
    let verification = serde_json::json!({
        "cef": runtime::CEF_VERSION,
        "mode": if lifecycle_mode { "native CEF page lifecycle and CPU OnPaint isolation fixture" } else if fixture { "native CEF CPU OnPaint pixel fixture" } else { "native CEF CPU OnPaint supplied URL" },
        "settleSeconds": settle.as_secs(), "image": result.as_ref().ok(),
        "error": result.as_ref().err().map(ToString::to_string),
        "document": {
            "loading": page.loading, "completedLoads": page.completed_loads,
            "httpStatus": page.http_status, "title": page.title,
            "loadErrors": page.load_errors, "console": page.console,
        },
        "cache": cache, "onBeforeCloseAndShutdown": matches!(&shutdown, Ok(true)),
        "shutdownError": shutdown.as_ref().err(),
        "lifecycle": lifecycle_mode.then(|| serde_json::json!({
            "checks": lifecycle_evidence, "beforeShutdown": shutdown_start.as_ref().map(lifecycle::status_json),
            "runtimeReleased": runtime_released,
        })),
    });
    std::fs::write(&report, serde_json::to_vec_pretty(&verification)?)?;
    result?;
    if !shutdown? || !runtime_released {
        return Err("CEF browser shutdown timed out".into());
    }
    println!(
        "CEF {} loaded and painted; PNG {}, report {}; OnBeforeClose and cef_shutdown completed",
        runtime::CEF_VERSION,
        output.display(),
        report.display()
    );
    Ok(())
}
