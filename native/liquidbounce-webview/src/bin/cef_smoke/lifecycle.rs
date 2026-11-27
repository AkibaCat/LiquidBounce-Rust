//! Real CEF lifecycle checks. No HTTP server, replacement renderer or mocked browser is used.
use liquidbounce_webview::{
    input::{Input, InputQueue, PageTarget},
    paint::Mailbox,
    runtime::{self, PageState, PageStatus},
};
use serde_json::{Value, json};
use std::{
    error::Error,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const RED: [u8; 4] = [0, 0, 255, 255];
const BLUE: [u8; 4] = [255, 0, 0, 255];
const SHARED_TITLE: &str = "CEF lifecycle / shared";
const CLICK_TITLE: &str = "CEF lifecycle / clickgui";

fn data_url(html: &str) -> String {
    let encoded: String = html
        .as_bytes()
        .iter()
        .map(|byte| format!("%{byte:02X}"))
        .collect();
    format!("data:text/html;charset=utf-8,{encoded}")
}

pub fn initial_url() -> String {
    data_url(include_str!("../../../fixtures/lifecycle-shared.html"))
}
fn click_url() -> String {
    data_url(include_str!("../../../fixtures/lifecycle-clickgui.html"))
}

pub fn status_json(status: &PageStatus) -> Value {
    json!({
        "active": format!("{:?}", status.active),
        "sharedOpen": status.shared_open, "clickGuiOpen": status.click_gui_open,
        "sharedBrowserId": status.shared_browser_id, "clickGuiBrowserId": status.click_gui_browser_id,
        "retiredPages": status.retired_pages, "createdPages": status.created_pages,
        "closedPages": status.closed_pages, "configuredFps": status.fps,
        "sharedNativeFps": status.shared_fps, "clickGuiNativeFps": status.click_gui_fps,
    })
}

fn status() -> Result<PageStatus> {
    runtime::page_status_current_thread().ok_or_else(|| "CEF lifecycle runtime disappeared".into())
}

fn sample(mailbox: &Mailbox) -> Option<[u8; 4]> {
    let image = mailbox.snapshot().view?;
    let offset = ((image.height / 2) * image.width + image.width / 2) as usize * 4;
    image.bgra.get(offset..offset + 4)?.try_into().ok()
}

fn require(condition: bool, stage: &str, detail: impl std::fmt::Debug) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(format!("CEF lifecycle {stage} failed: {detail:?}").into())
    }
}

fn push(input: &InputQueue, event: Input) -> Result<()> {
    input.push(event).map_err(Into::into)
}

fn pump_until(stage: &str, mut ready: impl FnMut() -> bool, mailbox: &Mailbox) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        runtime::pump_current_thread()?;
        if ready() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "CEF lifecycle {stage} timeout; status={:?}, document={:?}, pixel={:?}",
                runtime::page_status_current_thread(),
                runtime::page_state_current_thread(),
                sample(mailbox)
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn wait_document(
    stage: &str,
    title: &str,
    pixel: [u8; 4],
    loads: u64,
    mailbox: &Mailbox,
) -> Result<PageState> {
    pump_until(
        stage,
        || {
            runtime::page_state_current_thread().is_some_and(|page| {
                !page.loading
                    && page.completed_loads >= loads
                    && page.title == title
                    && sample(mailbox) == Some(pixel)
            })
        },
        mailbox,
    )?;
    let page = runtime::page_state_current_thread().ok_or("CEF document disappeared")?;
    let errors: Vec<_> = page
        .load_errors
        .iter()
        .filter(|error| !error.contains("ABORTED"))
        .collect();
    require(errors.is_empty(), stage, errors)?;
    require(
        !page.http_status.is_some_and(|code| code >= 400),
        stage,
        page.http_status,
    )?;
    Ok(page)
}

fn record(evidence: &mut Vec<Value>, check: &str, mailbox: &Mailbox, extra: Value) -> Result<()> {
    evidence.push(json!({ "check": check, "status": status_json(&status()?),
        "centerBgra": sample(mailbox), "detail": extra }));
    Ok(())
}

pub fn run(mailbox: &Mailbox, input: &InputQueue, evidence: &mut Vec<Value>) -> Result<()> {
    let shared = wait_document("initial shared page", SHARED_TITLE, RED, 1, mailbox)?;
    let first = status()?;
    let shared_id = first
        .shared_browser_id
        .ok_or("shared browser was not created")?;
    require(
        first.active == PageTarget::Shared && first.click_gui_browser_id.is_none(),
        "initial page selection",
        &first,
    )?;
    record(
        evidence,
        "initial shared document painted",
        mailbox,
        json!({"completedLoads":shared.completed_loads}),
    )?;

    push(input, Input::EnsurePage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let prewarmed = status()?;
    let click_id = prewarmed
        .click_gui_browser_id
        .ok_or("EnsurePage did not create ClickGUI")?;
    require(
        prewarmed.active == PageTarget::Shared
            && prewarmed.shared_browser_id == Some(shared_id)
            && click_id != shared_id
            && sample(mailbox) == Some(RED),
        "EnsurePage preserves selection and pixels",
        &prewarmed,
    )?;
    record(
        evidence,
        "EnsurePage created a distinct hidden browser without selecting it",
        mailbox,
        json!({}),
    )?;

    push(input, Input::SelectPage(PageTarget::ClickGui))?;
    wait_document(
        "prewarmed ClickGUI initial document",
        SHARED_TITLE,
        RED,
        1,
        mailbox,
    )?;
    push(input, Input::Navigate(click_url()))?;
    let click = wait_document("first clickgui page", CLICK_TITLE, BLUE, 1, mailbox)?;
    require(
        status()?.click_gui_browser_id == Some(click_id),
        "SelectPage reuses prewarmed browser",
        status()?,
    )?;
    record(
        evidence,
        "SelectPage reused the prewarmed browser and painted its own document",
        mailbox,
        json!({"completedLoads":click.completed_loads}),
    )?;

    push(input, Input::SelectPage(PageTarget::Shared))?;
    runtime::pump_current_thread()?;
    let restored_shared = status()?;
    require(
        restored_shared.active == PageTarget::Shared
            && restored_shared.shared_browser_id == Some(shared_id)
            && sample(mailbox) == Some(RED),
        "immediate shared cache restoration",
        &restored_shared,
    )?;
    record(
        evidence,
        "switching restored the shared CPU surface before another message-loop paint",
        mailbox,
        json!({}),
    )?;

    push(input, Input::SelectPage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let restored_click = status()?;
    let current_click =
        runtime::page_state_current_thread().ok_or("cached clickgui document missing")?;
    require(
        restored_click.active == PageTarget::ClickGui
            && restored_click.click_gui_browser_id == Some(click_id)
            && sample(mailbox) == Some(BLUE)
            && current_click.completed_loads == click.completed_loads,
        "immediate ClickGUI cache restoration without a reload",
        (&restored_click, &current_click),
    )?;
    record(
        evidence,
        "cached ClickGUI kept its browser, load count and pixels",
        mailbox,
        json!({}),
    )?;

    push(input, Input::ReloadPage(PageTarget::Shared))?;
    let isolation_deadline = Instant::now() + Duration::from_millis(500);
    while Instant::now() < isolation_deadline {
        runtime::pump_current_thread()?;
        require(
            status()?.active == PageTarget::ClickGui && sample(mailbox) == Some(BLUE),
            "background reload paint isolation",
            (status()?, sample(mailbox)),
        )?;
        std::thread::sleep(Duration::from_millis(5));
    }
    record(
        evidence,
        "reloading the background shared browser did not overwrite ClickGUI pixels",
        mailbox,
        json!({"observedMillis":500}),
    )?;
    push(input, Input::SelectPage(PageTarget::Shared))?;
    let reloaded_shared = wait_document(
        "background reload completion",
        SHARED_TITLE,
        RED,
        shared.completed_loads + 1,
        mailbox,
    )?;
    require(
        status()?.shared_browser_id == Some(shared_id),
        "ReloadPage keeps native browser identity",
        status()?,
    )?;
    record(
        evidence,
        "background ReloadPage completed on its own document",
        mailbox,
        json!({"completedLoads":reloaded_shared.completed_loads}),
    )?;

    push(input, Input::SelectPage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    push(input, Input::ClosePage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let closed = status()?;
    require(
        closed.active == PageTarget::Shared
            && closed.shared_browser_id == Some(shared_id)
            && closed.click_gui_browser_id.is_none()
            && sample(mailbox) == Some(RED),
        "ClosePage returns to Shared",
        &closed,
    )?;
    pump_until(
        "closed ClickGUI OnBeforeClose",
        || {
            runtime::page_status_current_thread()
                .is_some_and(|s| s.retired_pages == 0 && s.closed_pages >= first.closed_pages + 1)
        },
        mailbox,
    )?;
    record(
        evidence,
        "ClosePage destroyed ClickGUI and observed OnBeforeClose",
        mailbox,
        json!({"closedBrowserId":click_id}),
    )?;

    let before_missing_reload = status()?;
    push(input, Input::ReloadPage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let missing_reload = status()?;
    require(
        missing_reload.click_gui_browser_id.is_none()
            && missing_reload.created_pages == before_missing_reload.created_pages,
        "ReloadPage does not create absent pages",
        &missing_reload,
    )?;
    record(
        evidence,
        "ReloadPage ignored an absent ClickGUI browser",
        mailbox,
        json!({}),
    )?;

    push(input, Input::EnsurePage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let reopened = status()?;
    let reopened_click_id = reopened
        .click_gui_browser_id
        .ok_or("ClickGUI did not reopen")?;
    require(
        reopened.active == PageTarget::Shared
            && reopened_click_id != click_id
            && reopened_click_id != shared_id,
        "EnsurePage after ClosePage creates a new browser",
        &reopened,
    )?;
    push(input, Input::SelectPage(PageTarget::ClickGui))?;
    wait_document(
        "reopened ClickGUI initial document",
        SHARED_TITLE,
        RED,
        1,
        mailbox,
    )?;
    push(input, Input::Navigate(click_url()))?;
    wait_document("reopened ClickGUI", CLICK_TITLE, BLUE, 1, mailbox)?;
    record(
        evidence,
        "reopening created a new native ClickGUI browser",
        mailbox,
        json!({"previousBrowserId":click_id}),
    )?;

    const FPS: u32 = 37;
    push(input, Input::RestartPages { fps: FPS })?;
    runtime::pump_current_thread()?;
    let restarted = status()?;
    let next_shared = restarted
        .shared_browser_id
        .ok_or("RestartPages lost Shared")?;
    let next_click = restarted
        .click_gui_browser_id
        .ok_or("RestartPages lost ClickGUI")?;
    require(
        restarted.active == PageTarget::ClickGui
            && restarted.fps == FPS
            && restarted.shared_fps == Some(FPS)
            && restarted.click_gui_fps == Some(FPS)
            && next_shared != shared_id
            && next_click != reopened_click_id
            && next_shared != next_click,
        "RestartPages replaces both native browser IDs and preserves selection/fps",
        &restarted,
    )?;
    wait_document(
        "restarted ClickGUI preserved URL",
        CLICK_TITLE,
        BLUE,
        1,
        mailbox,
    )?;
    pump_until(
        "restarted pages OnBeforeClose",
        || {
            runtime::page_status_current_thread()
                .is_some_and(|s| s.retired_pages == 0 && s.closed_pages >= first.closed_pages + 3)
        },
        mailbox,
    )?;
    record(
        evidence,
        "RestartPages recreated both browsers and kept ClickGUI URL and selection",
        mailbox,
        json!({"previousSharedId":shared_id,"previousClickGuiId":reopened_click_id,"configuredFps":FPS}),
    )?;

    push(input, Input::SelectPage(PageTarget::Shared))?;
    wait_document(
        "restarted Shared preserved URL",
        SHARED_TITLE,
        RED,
        1,
        mailbox,
    )?;
    require(
        status()?.shared_browser_id == Some(next_shared),
        "restarted Shared identity",
        status()?,
    )?;
    push(input, Input::SelectPage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    require(
        status()?.click_gui_browser_id == Some(next_click) && sample(mailbox) == Some(BLUE),
        "restarted cached surface isolation",
        (status()?, sample(mailbox)),
    )?;
    record(
        evidence,
        "both recreated documents retained independent surfaces",
        mailbox,
        json!({"capturedPage":"ClickGui"}),
    )?;
    Ok(())
}

/// Leave one asynchronously closing page alongside two live pages. The caller must immediately
/// use shutdown_current_thread so its result covers both live and retired browser lifetimes.
pub fn prepare_shutdown(
    mailbox: &Mailbox,
    input: &InputQueue,
    evidence: &mut Vec<Value>,
) -> Result<()> {
    let before = status()?;
    push(input, Input::ClosePage(PageTarget::ClickGui))?;
    push(input, Input::EnsurePage(PageTarget::ClickGui))?;
    runtime::pump_current_thread()?;
    let pending = status()?;
    require(
        pending.shared_open
            && pending.click_gui_open
            && pending.retired_pages >= 1
            && pending.click_gui_browser_id != before.click_gui_browser_id
            && pending.active == PageTarget::Shared,
        "shutdown fixture includes live and retired pages",
        &pending,
    )?;
    record(
        evidence,
        "shutdown began with two live browsers and a retired browser",
        mailbox,
        json!({}),
    )
}
