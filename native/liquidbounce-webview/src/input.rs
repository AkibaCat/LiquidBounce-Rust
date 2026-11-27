use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

/// Independent Chromium documents selected by the native screen policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageTarget {
    #[default]
    Shared,
    ClickGui,
}

/// Modifiers are CEF event flags (shift=2, ctrl=4, alt=8, left=16, middle=32, right=64).
#[derive(Clone, Debug)]
pub enum Input {
    MouseMove {
        x: i32,
        y: i32,
        modifiers: u32,
        leave: bool,
    },
    MouseButton {
        x: i32,
        y: i32,
        modifiers: u32,
        button: Button,
        up: bool,
        clicks: u8,
    },
    Wheel {
        x: i32,
        y: i32,
        modifiers: u32,
        delta_x: i32,
        delta_y: i32,
    },
    Key {
        virtual_key: i32,
        native_code: i32,
        modifiers: u32,
        up: bool,
        system: bool,
    },
    Text {
        utf16: Vec<u16>,
        modifiers: u32,
    },
    Focus(bool),
    Visible(bool),
    Navigate(String),
    Reload,
    /// Request a fresh main-view paint after a route acknowledges its DOM.
    Repaint,
    /// Lazily create or reuse this document and make it the input/paint recipient.
    SelectPage(PageTarget),
    /// Preload this document without changing the active recipient.
    EnsurePage(PageTarget),
    /// Destroy this document; closing the active ClickGUI returns to Shared.
    ClosePage(PageTarget),
    /// Reload only if this document already exists.
    ReloadPage(PageTarget),
    /// Recreate every existing document using the new offscreen paint rate.
    RestartPages {
        fps: u32,
    },
    /// Independent input-free HUD document. Hidden pages retain their live DOM.
    HudOverlay {
        url: String,
        fps: u32,
        visible: bool,
        close: bool,
        generation: u64,
    },
}

/// Horizon screen mouse coordinates are GUI pixels; CEF's viewport uses physical pixels.
pub fn gui_to_browser(x: f32, y: f32, gui_scale: f32) -> (i32, i32) {
    (
        (x * gui_scale).round() as i32,
        (y * gui_scale).round() as i32,
    )
}

pub fn text(text: &str, modifiers: u32) -> Input {
    Input::Text {
        utf16: text.encode_utf16().collect(),
        modifiers,
    }
}

#[derive(Clone, Default)]
pub struct InputQueue(Arc<Mutex<VecDeque<Input>>>);

impl InputQueue {
    pub fn push(&self, event: Input) -> Result<(), &'static str> {
        let mut queue = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if queue.len() >= 4096 {
            return Err("CEF input queue is full");
        }
        queue.push_back(event);
        Ok(())
    }
    #[cfg(feature = "cef-runtime")]
    pub(crate) fn drain(&self) -> Vec<Input> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_input_keeps_surrogate_pairs_and_gui_coordinates_scale_once() {
        let Input::Text { utf16, modifiers } = text("中😀", 2) else {
            unreachable!()
        };
        assert_eq!(utf16, [0x4e2d, 0xd83d, 0xde00]);
        assert_eq!(modifiers, 2);
        assert_eq!(gui_to_browser(15.0, 7.5, 2.0), (30, 15));
    }
}
