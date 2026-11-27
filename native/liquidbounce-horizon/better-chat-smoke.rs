#[cfg(test)]
mod runtime_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        time::{Duration, Instant},
    };
    #[test]
    fn real_http_provider_worker_is_nonblocking_and_cache_is_shared() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (ready, waiting) = mpsc::channel();
        let (release, proceed) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            let mut data = [0u8; 2048];
            loop {
                let n = stream.read(&mut data).unwrap();
                assert!(n > 0);
                request.extend_from_slice(&data[..n]);
                if let Some(start) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..start]);
                    let length = headers
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(str::trim)
                                .and_then(|v| v.parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if request.len() >= start + 4 + length {
                        break;
                    }
                }
            }
            let request = String::from_utf8(request).unwrap();
            assert!(request.starts_with("POST /translate HTTP/1.1"));
            assert!(request.contains("private-key"));
            assert!(request.contains("\"target\":\"zt\""));
            ready.send(()).unwrap();
            // The main-thread poll must return before this response is released.
            proceed.recv_timeout(Duration::from_secs(3)).unwrap();
            let body = r#"{"translatedText":"你好","detectedLanguage":{"language":"en"}}"#;
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(reply.as_bytes()).unwrap();
        });
        let mut r = BetterChatRuntime::default();
        r.translation = TranslationSettings {
            provider: 1,
            base_url: format!("http://{address}"),
            api_key: "private-key".into(),
            language_code: "zh-Hant-TW".into(),
        };
        let requested_settings = r.translation.clone();
        assert!(r.translate("Hello".into(), 0).is_none());
        // Changing settings while queued must not alter the request snapshot.
        r.translation.language_code = "en".into();
        assert!(r.poll(0).is_empty());
        assert_eq!(r.workers.len(), 1);
        waiting.recv_timeout(Duration::from_secs(3)).unwrap();
        release.send(()).unwrap();
        let start = Instant::now();
        let mut output = vec![];
        while start.elapsed() < Duration::from_secs(3) {
            output.extend(r.poll(100));
            if !output.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        server.join().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].translation, "你好");
        assert_eq!(output[0].to, "zh-Hant-TW");
        r.translation = requested_settings;
        assert_eq!(r.translate("Hello".into(), 200), Some(output.remove(0)));
        assert!(r.pending.is_empty());
    }
    #[test]
    fn cancel_drops_pending_work_and_provider_change_clears_cache() {
        let mut r = BetterChatRuntime::default();
        let s = r.translation.clone();
        let result = Translation {
            origin: "hello".into(),
            translation: "你好".into(),
            from: "en".into(),
            to: "zh".into(),
        };
        r.provider_changed();
        r.cache.put(s.clone(), result, 0);
        assert!(r.cache.get(&s, "hello", 1).is_some());
        r.translation.provider = 1;
        r.provider_changed();
        assert!(r.cache.get(&s, "hello", 1).is_none());
        r.pending.push_back((s, "queued".into()));
        r.cancel();
        assert!(r.pending.is_empty());
        assert!(r.workers.is_empty());
        assert_eq!(
            request(
                &TranslationSettings {
                    provider: 1,
                    base_url: ":invalid".into(),
                    ..Default::default()
                },
                "hello"
            ),
            Err("Invalid translator URL")
        );
    }
}
