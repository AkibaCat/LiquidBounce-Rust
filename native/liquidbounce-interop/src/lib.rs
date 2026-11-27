// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce's ClientInteropServer, AuthPlugin and WebSocketSessionManager.
// Copyright (c) 2015-2026 CCBlueX. https://github.com/CCBlueX/LiquidBounce

//! Real localhost transport for the unchanged LiquidBounce Web theme.
//! The game backend supplies every API result and executes state changes at the
//! actual game-thread boundary. This crate invents neither modules nor world data.

pub mod desktop;
pub mod localization;
pub mod main_thread;
pub mod protocol;
pub mod ui;

pub use axum::http::{Method, StatusCode};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{
        Request, State,
        ws::{Message, WebSocket, WebSocketUpgrade, rejection::WebSocketUpgradeRejection},
    },
    http::{HeaderMap, HeaderValue, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::sync::{broadcast, oneshot};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone, Debug)]
pub struct ApiRequest {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    /// Original request bytes: config deserialization is the backend's job.
    pub body: Bytes,
}

pub struct ApiResponse {
    pub status: StatusCode,
    pub body: Option<Value>,
}

impl ApiResponse {
    pub fn json(body: Value) -> Self {
        Self {
            status: StatusCode::OK,
            body: Some(body),
        }
    }
    pub fn no_content() -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            body: None,
        }
    }
    pub fn error(status: StatusCode, reason: impl Into<String>) -> Self {
        Self {
            status,
            body: Some(json!({"reason": reason.into()})),
        }
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self.body {
            Some(body) => (self.status, Json(body)).into_response(),
            None => self.status.into_response(),
        }
    }
}

/// A synchronous backend called from a worker, never from the game/render loop.
/// Queue mutations to the real client thread and return its actual response.
pub trait Backend: Send + Sync + 'static {
    fn request(&self, request: ApiRequest) -> ApiResponse;
}

#[derive(Clone, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub author: String,
}

pub struct ServerConfig {
    pub port: u16,
    pub auth_code: String,
    pub theme_root: PathBuf,
    pub info: ServerInfo,
    /// Corresponds to ThemeManager.isThemeExternal in the original middleware.
    pub external_theme: bool,
}

impl ServerConfig {
    pub fn original_theme(theme_root: PathBuf, info: ServerInfo) -> io::Result<Self> {
        // Same 16-character secure alphanumeric token alphabet as upstream.
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut code = String::with_capacity(16);
        while code.len() < 16 {
            let mut entropy = [0u8; 32];
            getrandom::fill(&mut entropy).map_err(io::Error::other)?;
            for byte in entropy {
                if byte < 248 && code.len() < 16 {
                    code.push(char::from(alphabet[usize::from(byte) % alphabet.len()]));
                }
            }
        }
        Ok(Self {
            port: 0,
            auth_code: code,
            theme_root,
            info,
            external_theme: false,
        })
    }
}

#[derive(Clone)]
struct Shared {
    backend: Arc<dyn Backend>,
    config: Arc<ServerConfig>,
    events: broadcast::Sender<String>,
}

pub struct Server {
    address: SocketAddr,
    auth_code: String,
    events: broadcast::Sender<String>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    pub fn start(config: ServerConfig, backend: Arc<dyn Backend>) -> io::Result<Self> {
        if config.auth_code.is_empty()
            || !config.auth_code.bytes().all(|v| v.is_ascii_alphanumeric())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid authentication code",
            ));
        }
        if !config.theme_root.join("index.html").is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "built original theme index.html missing",
            ));
        }
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, config.port))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let auth_code = config.auth_code.clone();
        let (events, _) = broadcast::channel(1024);
        let (stop, stopping) = oneshot::channel();
        let shared = Shared {
            backend,
            config: Arc::new(config),
            events: events.clone(),
        };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        let thread = thread::Builder::new()
            .name("LiquidBounce interop".into())
            .spawn(move || {
                runtime.block_on(async move {
                    let listener = tokio::net::TcpListener::from_std(listener)
                        .expect("validated TCP listener");
                    let app = router(shared);
                    // Dropping the server future stops its open connections on shutdown.
                    tokio::select! {
                        _ = axum::serve(listener, app) => {},
                        _ = stopping => {},
                    }
                });
                runtime.shutdown_timeout(Duration::from_secs(2));
            })?;
        Ok(Self {
            address,
            auth_code,
            events,
            stop: Some(stop),
            thread: Some(thread),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }
    /// Authenticate the page first; its root WebSocket uses the resulting HttpOnly cookie.
    pub fn theme_url(&self) -> String {
        format!(
            "http://{}/resource/liquidbounce/?lb_code={}",
            self.address, self.auth_code
        )
    }
    pub fn broadcast(&self, name: &str, event: &Value) {
        let _ = self
            .events
            .send(json!({"name": name, "event": event}).to_string());
    }
    pub fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

fn query(uri: &Uri) -> HashMap<String, String> {
    serde_urlencoded::from_str(uri.query().unwrap_or_default()).unwrap_or_default()
}

fn cookie_authenticated(headers: &HeaderMap, code: &str) -> bool {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|value| {
            value
                .split(';')
                .filter_map(|v| v.trim().split_once('='))
                .any(|(name, value)| name == "lb_auth" && value == code)
        })
}

async fn authenticate(State(shared): State<Shared>, request: Request, next: Next) -> Response {
    let by_code = query(request.uri()).get("lb_code") == Some(&shared.config.auth_code);
    if !by_code
        && !shared.config.external_theme
        && !cookie_authenticated(request.headers(), &shared.config.auth_code)
    {
        return ApiResponse::error(StatusCode::UNAUTHORIZED, "Authentication required")
            .into_response();
    }
    let mut response = next.run(request).await;
    if by_code {
        let value = format!("lb_auth={}; Path=/; HttpOnly", shared.config.auth_code);
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&value).expect("validated auth code"),
        );
    }
    response
}

fn router(shared: Shared) -> Router {
    let files = ServeDir::new(&shared.config.theme_root)
        .not_found_service(ServeFile::new(shared.config.theme_root.join("index.html")));
    Router::new()
        .route("/", get(root))
        .route("/api/v1/{*path}", get(api).post(api).put(api).delete(api))
        .nest_service("/resource/liquidbounce", files)
        .layer(middleware::from_fn_with_state(shared.clone(), authenticate))
        .with_state(shared)
}

async fn root(
    State(shared): State<Shared>,
    headers: HeaderMap,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Response {
    if let Ok(upgrade) = upgrade {
        let authenticated = shared.config.external_theme
            || cookie_authenticated(&headers, &shared.config.auth_code);
        return upgrade
            .on_upgrade(move |socket| websocket(socket, shared, authenticated))
            .into_response();
    }
    Json(shared.config.info.clone()).into_response()
}

async fn api(State(shared): State<Shared>, method: Method, uri: Uri, body: Bytes) -> Response {
    let request = ApiRequest {
        method,
        path: uri.path().into(),
        query: query(&uri),
        body,
    };
    match tokio::task::spawn_blocking(move || shared.backend.request(request)).await {
        Ok(reply) => reply.into_response(),
        Err(_) => ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, "API backend failed")
            .into_response(),
    }
}

async fn websocket(mut socket: WebSocket, shared: Shared, authenticated: bool) {
    if !authenticated {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }
    let mut events = shared.events.subscribe();
    let mut ping = tokio::time::interval(Duration::from_secs(15));
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    ping.tick().await;
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                _ => {}, // Original endpoint consumes no application messages.
            },
            message = events.recv() => match message {
                Ok(message) => if socket.send(Message::Text(message.into())).await.is_err() { break; },
                // Disconnect on overflow: silently dropping state events would desynchronize the theme.
                Err(_) => break,
            },
            _ = ping.tick() => if socket.send(Message::Ping(Bytes::new())).await.is_err() { break; },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    struct CapturingBackend;
    impl Backend for CapturingBackend {
        fn request(&self, request: ApiRequest) -> ApiResponse {
            ApiResponse::json(
                json!({"method": request.method.as_str(), "path": request.path,
                                    "query": request.query, "body": String::from_utf8(request.body.to_vec()).unwrap()}),
            )
        }
    }
    fn shared() -> Shared {
        Shared {
            backend: Arc::new(CapturingBackend),
            config: Arc::new(ServerConfig {
                port: 0,
                auth_code: "SecretToken123456".into(),
                theme_root: PathBuf::new(),
                info: ServerInfo {
                    name: "LiquidBounce".into(),
                    version: "0.43.1".into(),
                    author: "CCBlueX".into(),
                },
                external_theme: false,
            }),
            events: broadcast::channel(8).0,
        }
    }
    #[tokio::test]
    async fn original_authentication_bootstraps_cookie_and_guards_api() {
        let app = router(shared());
        let denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/client/modules")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            denied.into_body().collect().await.unwrap().to_bytes(),
            r#"{"reason":"Authentication required"}"#
        );
        let bootstrap = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/?lb_code=SecretToken123456")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bootstrap.status(), StatusCode::OK);
        assert_eq!(
            bootstrap.headers()[header::SET_COOKIE],
            "lb_auth=SecretToken123456; Path=/; HttpOnly"
        );
        let allowed = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/client/modules")
                    .header(header::COOKIE, "other=a; lb_auth=SecretToken123456")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(allowed.status(), StatusCode::OK);
    }
    #[tokio::test]
    async fn api_preserves_method_config_bytes_and_decoded_module_name() {
        let response = router(shared())
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/v1/client/modules/settings?name=No%20Fov")
                    .header(header::COOKIE, "lb_auth=SecretToken123456")
                    .body(Body::from("{ \"name\": \"NoFov\", \"value\": [] }\n"))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["method"], "PUT");
        assert_eq!(value["path"], "/api/v1/client/modules/settings");
        assert_eq!(value["query"]["name"], "No Fov");
        assert_eq!(value["body"], "{ \"name\": \"NoFov\", \"value\": [] }\n");
    }

    #[tokio::test]
    async fn live_websocket_requires_cookie_and_delivers_original_event_envelope() {
        use futures_util::StreamExt;
        use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};
        let folder = tempfile::tempdir().unwrap();
        std::fs::write(folder.path().join("index.html"), "<html></html>").unwrap();
        let config = ServerConfig {
            port: 0,
            auth_code: "SecretToken123456".into(),
            theme_root: folder.path().into(),
            info: shared().config.info.clone(),
            external_theme: false,
        };
        let server = Server::start(config, Arc::new(CapturingBackend)).unwrap();
        let query_url = format!("ws://{}/?lb_code=SecretToken123456", server.address());
        let (mut query_socket, response) = connect_async(query_url).await.unwrap();
        assert!(response.headers().contains_key(header::SET_COOKIE));
        let close = tokio::time::timeout(Duration::from_secs(2), query_socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(
            close,
            tokio_tungstenite::tungstenite::Message::Close(_)
        ));
        let mut request = format!("ws://{}/", server.address())
            .into_client_request()
            .unwrap();
        request.headers_mut().insert(
            header::COOKIE,
            HeaderValue::from_static("lb_auth=SecretToken123456"),
        );
        let (mut socket, _) = connect_async(request).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while server.events.receiver_count() == 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        server.broadcast(
            "moduleToggle",
            &json!({"moduleName": "Zoom", "hidden": false, "enabled": true}),
        );
        let frame = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let value: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        assert_eq!(
            value,
            json!({"name": "moduleToggle", "event": {"moduleName": "Zoom", "hidden": false, "enabled": true}})
        );
        socket.close(None).await.unwrap();
        server.stop();
    }
}
