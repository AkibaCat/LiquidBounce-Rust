// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    random::{random_i32_inclusive, random_unit_f32},
    spammer::{self, FileIndex, Spammer},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{self, Receiver, SyncSender},
    },
    time::Duration,
};
#[derive(Clone)]
struct Context {
    settings: Spammer,
    players: Vec<String>,
    root: PathBuf,
    stopped: bool,
}
enum Output {
    Send(String),
    Error(String),
    TooLong,
}
struct Worker {
    shared: Arc<(Mutex<Context>, Condvar)>,
    output: Receiver<Output>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).stopped = true;
        wake.notify_all();
    }
}
#[derive(Default)]
pub struct SpammerRuntime {
    pub settings: Spammer,
    worker: Option<Worker>,
}
impl SpammerRuntime {
    pub fn cancel(&mut self) {
        self.worker = None;
    }
    fn update(&mut self, root: PathBuf, players: Vec<String>) -> Vec<Output> {
        let context = Context {
            settings: self.settings.clone(),
            players,
            root,
            stopped: false,
        };
        if let Some(worker) = &self.worker {
            worker
                .shared
                .0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone_from(&context);
        } else {
            let shared = Arc::new((Mutex::new(context), Condvar::new()));
            let state = shared.clone();
            let (tx, rx) = mpsc::sync_channel(1024);
            if let Err(error) = std::thread::Builder::new()
                .name("LiquidBounce-Spammer".into())
                .spawn(move || run(state, tx))
            {
                return vec![Output::Error(error.to_string())];
            }
            self.worker = Some(Worker { shared, output: rx });
        }
        self.worker.as_ref().unwrap().output.try_iter().collect()
    }
}
fn run(shared: Arc<(Mutex<Context>, Condvar)>, output: SyncSender<Output>) {
    let mut file = FileIndex::default();
    let mut setting_linear = 0usize;
    let mut file_linear = 0usize;
    loop {
        let context = shared.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if context.stopped {
            return;
        }
        for _ in 0..random_i32_inclusive(context.settings.mps[0], context.settings.mps[1]).max(0) {
            let current = shared.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
            if current.stopped {
                return;
            }
            let s = &current.settings;
            let next = if s.source == 0 {
                if s.messages.is_empty() {
                    Err("empty-source".to_owned())
                } else {
                    let index = if s.pattern == 1 {
                        let i = setting_linear;
                        setting_linear = setting_linear.wrapping_add(1);
                        i % s.messages.len()
                    } else {
                        random_i32_inclusive(0, s.messages.len() as i32 - 1) as usize
                    };
                    Ok(s.messages[index].clone())
                }
            } else {
                let p = PathBuf::from(&s.file);
                let p = if p.is_absolute() {
                    p
                } else {
                    current.root.join(p)
                };
                // Refresh the index before choosing a random line (including external file edits).
                match file.next(&p, 0) {
                    Err(e) => Err(e.to_string()),
                    Ok(_) => {
                        let index = if s.pattern == 1 {
                            let i = file_linear;
                            file_linear = file_linear.wrapping_add(1);
                            i
                        } else {
                            random_i32_inclusive(0, file.len() as i32 - 1) as usize
                        };
                        file.next(&p, index).map_err(|e| e.to_string())
                    }
                }
            };
            let event = match next {
                Ok(text) => {
                    let text = spammer::format_message(
                        s,
                        &text,
                        &current.players,
                        random_i32_inclusive,
                        random_unit_f32,
                    );
                    if spammer::utf16_len(&text) > 256 {
                        Output::TooLong
                    } else {
                        Output::Send(text)
                    }
                }
                Err(e) => Output::Error(e),
            };
            if output.send(event).is_err() {
                return;
            }
        }
        let context = shared.0.lock().unwrap_or_else(|e| e.into_inner());
        if context.stopped {
            return;
        }
        let range = context.settings.delay;
        let secs = f64::from(range[0] + random_unit_f32() * (range[1] - range[0]));
        let timeout = if secs.is_finite() && secs > 0.0 {
            Duration::from_secs_f64(secs.min(86400.0))
        } else {
            Duration::ZERO
        };
        let (guard, _) = shared
            .1
            .wait_timeout_while(context, timeout, |c| !c.stopped)
            .unwrap_or_else(|e| e.into_inner());
        if guard.stopped {
            return;
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn spammer_tick(&self, host: &mut dyn Host) {
        let players = host.client_spammer_players();
        let output = {
            let state = self.state();
            let root = state.config_root.clone();
            let mut modules = state.behaviors.borrow_mut();
            if modules.query_running(&state.registry, "Spammer") {
                modules.spammer.update(root, players)
            } else {
                modules.spammer.cancel();
                Vec::new()
            }
        };
        for event in output {
            match event {
                Output::Send(text) => host.client_spammer_send(&text),
                Output::TooLong => host.chat(&message(
                    "liquidbounce.module.spammer.messages.tooLong",
                    "Spammer message is too long! (Max 256 characters)",
                )),
                Output::Error(error) => {
                    let text = message(
                        "liquidbounce.module.spammer.messages.failed",
                        "Failed to get spammer message",
                    );
                    host.chat(&format!("{text}: {error}"));
                }
            }
        }
    }
}
fn message(key: &str, fallback: &str) -> String {
    #[cfg(feature = "webview")]
    if let Some(text) =
        liquidbounce_interop::localization::translate(&horizon_client::lang::code(), key)
    {
        return text.into();
    }
    let _ = key;
    fallback.into()
}
