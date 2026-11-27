// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;

impl LiquidBounceHooks {
    pub(crate) fn gui_closer_screen(&self, title: &str, cancel: &mut bool, print_title: &mut bool) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let running = modules.query_running(&state.registry, "GUICloser");
        let decision = modules.gui_closer.screen_opening(
            running, true, title,
        );
        *cancel |= decision.cancel;
        *print_title |= decision.print_title;
        drop(modules);
        #[cfg(feature = "webview")]
        {
            let mut state = state;
            state.probe(
                "clientGuiCloserScreen",
                serde_json::json!({"title":title,"running":running}),
                serde_json::json!({"cancel":*cancel,"printTitle":*print_title,"nullFilter":decision.null_filter}),
            );
        }
        if decision.null_filter {
            // The original EventManager catches Throwable around this handler.
            eprintln!("[LiquidBounce] GUICloser ScreenEvent: null Regex filter");
        }
    }
}
