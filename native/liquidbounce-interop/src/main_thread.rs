// SPDX-License-Identifier: GPL-3.0-or-later
//! Sends actual REST operations to the game thread instead of mutating from HTTP workers.
use crate::{ApiRequest, ApiResponse, Backend, StatusCode};
use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    time::{Duration, Instant},
};

pub struct QueuedBackend {
    sender: SyncSender<Pending>,
    timeout: Duration,
}

pub struct Inbox {
    receiver: Receiver<Pending>,
}

pub struct Pending {
    pub request: ApiRequest,
    expires: Instant,
    response: Option<SyncSender<ApiResponse>>,
}

impl QueuedBackend {
    pub fn channel(capacity: usize, timeout: Duration) -> (Arc<Self>, Inbox) {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        (Arc::new(Self { sender, timeout }), Inbox { receiver })
    }
}

impl Backend for QueuedBackend {
    fn request(&self, request: ApiRequest) -> ApiResponse {
        let (response, replies) = mpsc::sync_channel(1);
        let pending = Pending {
            request,
            expires: Instant::now() + self.timeout,
            response: Some(response),
        };
        match self.sender.try_send(pending) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                return ApiResponse::error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Game request queue is full",
                );
            }
            Err(TrySendError::Disconnected(_)) => {
                return ApiResponse::error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Game thread has stopped",
                );
            }
        }
        replies.recv_timeout(self.timeout).unwrap_or_else(|_| {
            ApiResponse::error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Game thread did not respond",
            )
        })
    }
}

impl Inbox {
    /// The actual game/render thread polls this from a real Host callback.
    /// Expired mutations are discarded, so a timed-out request cannot execute later.
    pub fn try_next(&self) -> Option<Pending> {
        while let Ok(pending) = self.receiver.try_recv() {
            if Instant::now() <= pending.expires {
                return Some(pending);
            }
        }
        None
    }
}

impl Pending {
    pub fn reply(mut self, value: ApiResponse) {
        if let Some(response) = self.response.take() {
            let _ = response.send(value);
        }
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        if let Some(response) = self.response.take() {
            let _ = response.send(ApiResponse::error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Game request was not executed",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Method;
    use std::{collections::HashMap, thread};
    fn request() -> ApiRequest {
        ApiRequest {
            method: Method::POST,
            path: "/api/v1/client/modules/toggle".into(),
            query: HashMap::new(),
            body: "{\"name\":\"Zoom\"}".into(),
        }
    }
    #[test]
    fn worker_waits_for_actual_main_thread_result() {
        let (backend, inbox) = QueuedBackend::channel(1, Duration::from_secs(2));
        let worker = thread::spawn(move || backend.request(request()));
        let until = Instant::now() + Duration::from_secs(2);
        let pending = loop {
            if let Some(pending) = inbox.try_next() {
                break pending;
            }
            assert!(Instant::now() < until);
            thread::yield_now();
        };
        assert!(!worker.is_finished());
        assert_eq!(pending.request.path, "/api/v1/client/modules/toggle");
        pending.reply(ApiResponse::no_content());
        assert_eq!(worker.join().unwrap().status, StatusCode::NO_CONTENT);
    }
    #[test]
    fn timed_out_mutation_never_executes_later() {
        let (backend, inbox) = QueuedBackend::channel(1, Duration::from_millis(5));
        let reply = backend.request(request());
        assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(inbox.try_next().is_none());
    }
}
