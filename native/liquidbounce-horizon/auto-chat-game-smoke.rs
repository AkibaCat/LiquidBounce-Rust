use auto_chat_game::{Settings, State};
use serde_json::{Value, json};
#[test]
fn buffering_gg_cooldown_and_cancel() {
    let mut s = State::default();
    let start = 1_000_000;
    s.message(start, "player-style non-trigger", |a, _| a);
    assert!(s.buffer.is_empty());
    s.message(start, "Chat Game:  1 + 1", |a, _| a);
    s.message(start + 150, "\tanswer\t now ", |a, _| a);
    assert!(s.question(start + 200).is_none());
    assert_eq!(
        s.question(start + 201).as_deref(),
        Some("Chat Game: 1 + 1 answer now")
    );
    s.message(start + 202, "Chat Game duplicate", |a, _| a);
    assert!(s.buffer.is_empty());
    s.message(start + 203, "Show some love by typing", |a, _| a);
    assert_eq!(s.due_gg(start + 1202), 0);
    assert_eq!(s.due_gg(start + 1203), 1);
    s.buffer.push("persistent".into());
    s.answer = Some((start + 9999, "old answer".into()));
    s.cancel();
    assert_eq!(s.buffer, ["persistent"]);
    assert!(!s.asking && s.answer.is_none() && s.gg.is_empty());
    s.buffer.clear();
    s.settings.cooldown = 0;
    s.settings.include_trigger = false;
    s.message(start + 1000, "Chat Game", |a, _| a);
    assert!(s.buffer.is_empty());
    s.message(
        start + 1001,
        "\u{00a0}A\u{00a0}\u{00a0}B\n\tC\u{00a0}",
        |a, _| a,
    );
    assert_eq!(
        s.question(start + 1201).as_deref(),
        Some("A\u{00a0}\u{00a0}B C")
    );
}
fn server(answer: &str, status: u16) -> (String, std::thread::JoinHandle<(String, Value)>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let answer = answer.to_owned();
    let worker = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut data = Vec::new();
        let mut buffer = [0u8; 4096];
        let end = loop {
            let n = socket.read(&mut buffer).unwrap();
            assert!(n > 0);
            data.extend_from_slice(&buffer[..n]);
            if let Some(i) = data.windows(4).position(|s| s == b"\r\n\r\n") {
                break i + 4;
            }
        };
        let headers = String::from_utf8(data[..end].to_vec()).unwrap();
        let length = headers
            .lines()
            .find_map(|line| {
                line.split_once(':')
                    .filter(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
                    .map(|(_, v)| v.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        while data.len() < end + length {
            let n = socket.read(&mut buffer).unwrap();
            assert!(n > 0);
            data.extend_from_slice(&buffer[..n]);
        }
        let body = serde_json::from_slice(&data[end..end + length]).unwrap();
        write!(socket,"HTTP/1.1 {status} Mock\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",answer.len()).unwrap();
        (headers, body)
    });
    (format!("http://{addr}/v1"), worker)
}
#[test]
fn real_http_protocols_and_provider_errors() {
    let (mut settings, question) = (Settings::default(), "2 + 2?");
    let (url, worker) = server(r#"{"choices":[{"message":{"content":" 4..."}}]}"#, 200);
    settings.base_url = url;
    settings.api_key = "local-smoke-secret".into();
    assert_eq!(request(&settings, question).unwrap(), " 4");
    let (headers, body) = worker.join().unwrap();
    assert!(headers.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(
        headers
            .to_ascii_lowercase()
            .contains("authorization: bearer local-smoke-secret")
    );
    assert_eq!(body["messages"][1]["content"], question);
    assert!(
        body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Minecraft")
    );
    let (url, worker) = server(
        r#"{"content":[{"type":"tool_use"},{"type":"text","text":"A"},{"type":"text","text":"B.."}]}"#,
        200,
    );
    settings.protocol = 1;
    settings.base_url = url;
    assert_eq!(request(&settings, question).unwrap(), "AB");
    let (headers, body) = worker.join().unwrap();
    assert!(headers.starts_with("POST /v1/messages HTTP/1.1"));
    let headers = headers.to_ascii_lowercase();
    assert!(
        headers.contains("anthropic-version: 2023-06-01")
            && headers.contains("x-api-key: local-smoke-secret")
    );
    assert_eq!(body["max_tokens"], 1024);
    assert!(body.get("system").is_some());
    let (url, worker) = server(r#"{"error":{"message":"bad local-smoke-secret"}}"#, 400);
    settings.base_url = url;
    let error = redact(request(&settings, question).unwrap_err(), &settings.api_key);
    worker.join().unwrap();
    assert!(error.contains("[redacted]") && !error.contains("local-smoke-secret"));
    assert!(wire::answer(1, &json!({"content":[{"type":"tool_use"}]})).is_err());
    assert!(wire::answer(0, &json!({"choices":[]})).is_err());
    assert_eq!(
        liquidbounce_behaviors::report_helper_format::format_command("/answer %1$s %% %<S", "ab")
            .unwrap(),
        "/answer ab % AB"
    );
}
