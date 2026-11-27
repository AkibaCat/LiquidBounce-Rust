use std::sync::{Arc, Mutex};

const MAX_PAINT_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Top-down BGRA8 with premultiplied colour, exactly as CEF OnPaint supplies it.
#[derive(Clone, Debug)]
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub bgra: Arc<[u8]>,
}

pub fn paint_length(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 {
        return None;
    }
    let length = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    (length <= MAX_PAINT_BYTES).then_some(length)
}

impl Pixels {
    pub fn copy(width: u32, height: u32, bgra: &[u8]) -> Result<Self, &'static str> {
        let length = paint_length(width, height).ok_or("invalid CEF paint dimensions")?;
        if length != bgra.len() {
            return Err("CEF paint buffer length does not match its dimensions");
        }
        Ok(Self {
            width,
            height,
            bgra: bgra.into(),
        })
    }
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub revision: u64,
    pub view: Option<Arc<Pixels>>,
    pub popup: Option<(Rect, Arc<Pixels>)>,
}

struct State {
    desired: [u32; 2],
    revision: u64,
    view: Option<Arc<Pixels>>,
    popup_rect: Rect,
    popup_visible: bool,
    popup: Option<Arc<Pixels>>,
    paint_count: u64,
    view_paint_count: u64,
}

/// Shared CPU images only; CEF callbacks never borrow a GPU encoder or hold a GPU lock.
pub struct Mailbox(Mutex<State>);

impl Mailbox {
    pub fn new(size: [u32; 2]) -> Self {
        Self(Mutex::new(State {
            desired: [size[0].max(1), size[1].max(1)],
            revision: 0,
            view: None,
            popup_rect: Rect::default(),
            popup_visible: false,
            popup: None,
            paint_count: 0,
            view_paint_count: 0,
        }))
    }

    pub fn resize(&self, size: [u32; 2]) {
        if paint_length(size[0], size[1]).is_some() {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).desired = size;
        }
    }

    pub fn desired_size(&self) -> [u32; 2] {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).desired
    }

    pub fn publish(&self, popup: bool, pixels: Pixels) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if popup {
            state.popup = Some(Arc::new(pixels));
        } else {
            state.view = Some(Arc::new(pixels));
            state.view_paint_count = state.view_paint_count.saturating_add(1);
        }
        state.revision = state.revision.wrapping_add(1);
        state.paint_count += 1;
    }

    pub fn popup_rect(&self, rect: Rect) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        state.popup_rect = rect;
        state.popup = None;
        state.revision = state.revision.wrapping_add(1);
    }

    pub fn popup_visible(&self, visible: bool) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        state.popup_visible = visible;
        if !visible {
            state.popup = None;
        }
        state.revision = state.revision.wrapping_add(1);
    }

    pub fn snapshot(&self) -> Snapshot {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        Snapshot {
            revision: state.revision,
            view: state.view.clone(),
            popup: state
                .popup_visible
                .then(|| state.popup.clone().map(|p| (state.popup_rect, p)))
                .flatten(),
        }
    }

    pub fn paint_count(&self) -> u64 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).paint_count
    }

    pub fn view_paint_count(&self) -> u64 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).view_paint_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_removal_restores_unchanged_view_and_preserves_top_down_pixels() {
        let mailbox = Mailbox::new([1, 2]);
        let view = [1, 2, 3, 255, 8, 9, 10, 128];
        mailbox.publish(false, Pixels::copy(1, 2, &view).unwrap());
        mailbox.popup_rect(Rect {
            x: -1,
            y: 1,
            width: 2,
            height: 1,
        });
        mailbox.popup_visible(true);
        mailbox.publish(true, Pixels::copy(2, 1, &[40; 8]).unwrap());
        assert!(mailbox.snapshot().popup.is_some());
        mailbox.popup_visible(false);
        let snapshot = mailbox.snapshot();
        assert!(snapshot.popup.is_none());
        assert_eq!(&*snapshot.view.unwrap().bgra, &view);
        assert!(Pixels::copy(1, 2, &[0; 4]).is_err());
        assert!(paint_length(u32::MAX, u32::MAX).is_none());
    }
}
