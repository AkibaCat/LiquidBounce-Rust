use crate::{
    compositor::{Compositor, Target},
    input::{Input, InputQueue},
    paint::Mailbox,
};
use horizon_hooks::gpu_ui;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Register one layer for its local player's browser before starting Horizon. The caller owns
/// the original screen-open/input hooks; this layer draws only the actual CEF OnPaint images.
pub struct Layer {
    pub mailbox: Arc<Mailbox>,
    pub input: InputQueue,
    pub seat: usize,
    visible: AtomicBool,
    compositing: AtomicBool,
    compositor: Mutex<Option<Compositor>>,
    hud_compositor: Mutex<Option<Compositor>>,
}

impl Layer {
    pub fn new(size: [u32; 2], seat: usize) -> Self {
        Self {
            mailbox: Arc::new(Mailbox::new(size)),
            input: InputQueue::default(),
            seat,
            visible: AtomicBool::new(false),
            compositing: AtomicBool::new(true),
            compositor: Mutex::new(None),
            hud_compositor: Mutex::new(None),
        }
    }
    pub fn set_visible(&self, visible: bool) {
        if self.visible.swap(visible, Ordering::AcqRel) == visible {
            return;
        }
        let _ = self.input.push(Input::Visible(visible));
        let _ = self.input.push(Input::Focus(visible));
    }
    /// Keep Chromium painting while withholding an unconfirmed menu surface.
    pub fn set_compositing(&self, enabled: bool) {
        self.compositing.store(enabled, Ordering::Release);
    }
    pub fn register(&'static self) -> Result<(), horizon_hooks::TooLate> {
        gpu_ui::register(self)
    }
}

impl gpu_ui::Layer for Layer {
    fn draw(&self, frame: &mut gpu_ui::Frame<'_>) {
        if frame.seat != self.seat {
            return;
        }
        let limit = frame.device.limits().max_texture_dimension_2d;
        if frame.size[0] <= limit && frame.size[1] <= limit {
            self.mailbox.resize(frame.size);
        }
        // Chromium advances on real frames only; frame generation uses the latest same GUI.
        #[cfg(feature = "cef-runtime")]
        if !frame.generated {
            if let Err(error) = crate::runtime::pump_current_thread() {
                eprintln!("LiquidBounce CEF: {error}");
            }
        }
        #[cfg(feature = "cef-runtime")]
        if let Some(snapshot) = crate::runtime::hud_snapshot_current_thread() {
            let mut cached = self
                .hud_compositor
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if cached.as_ref().is_none_or(|c| !c.uses_device(frame.device)) {
                *cached = Some(Compositor::new(frame.device));
            }
            let mut target = Target {
                device: frame.device,
                queue: frame.queue,
                encoder: &mut *frame.encoder,
                color: frame.color_view,
                color_format: frame.color_format,
                coverage: frame.coverage_view,
                size: frame.size,
            };
            if let Err(error) = cached.as_mut().unwrap().draw(&mut target, &snapshot) {
                eprintln!("LiquidBounce HUD CEF: {error}");
            }
        }
        if !self.visible.load(Ordering::Acquire) || !self.compositing.load(Ordering::Acquire) {
            return;
        }
        let snapshot = self.mailbox.snapshot();
        let mut cached = self.compositor.lock().unwrap_or_else(|e| e.into_inner());
        if cached.as_ref().is_none_or(|c| !c.uses_device(frame.device)) {
            *cached = Some(Compositor::new(frame.device));
        }
        let mut target = Target {
            device: frame.device,
            queue: frame.queue,
            encoder: &mut *frame.encoder,
            color: frame.color_view,
            color_format: frame.color_format,
            coverage: frame.coverage_view,
            size: frame.size,
        };
        if let Err(error) = cached.as_mut().unwrap().draw(&mut target, &snapshot) {
            eprintln!("LiquidBounce CEF: {error}");
        }
    }
}
