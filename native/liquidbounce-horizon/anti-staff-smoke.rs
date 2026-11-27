#[path = "../liquidbounce-behaviors/src/anti_staff.rs"]
mod anti_staff;
#[test]
fn cache_connection_and_names() {
    use anti_staff::*;
    assert_eq!(address("play.example.com:25565"), "example.com");
    assert_eq!(address("EXAMPLE.COM."), "example.com");
    assert_eq!(address("[2001:db8::1]:25565"), "2001:db8::1");
    assert_eq!(address("2001:db8::1"), "2001:db8::1");
    assert_eq!(address("999.999.999.999"), "999.999.999.999");
    let mut s = State::default();
    s.sync(true, true);
    assert_eq!(
        s.frame(Some("play.example.com:25565"), true, false),
        Some("example.com".into())
    );
    assert!(s.accept(
        s.generation,
        "example.com",
        staff_lines("Admin\r\nExact\r\n")
    ));
    assert!(s.is_staff("Admin"));
    assert!(!s.is_staff("admin"));
    assert_eq!(s.tab_staff().len(), 3);
    s.show_in_tab_list = false;
    assert!(s.tab_staff().is_empty());
    s.sync(false, false);
    assert!(!s.is_staff("Admin"));
    s.sync(true, true);
    assert!(s.frame(Some("example.com"), true, true).is_none());
    assert!(s.frame(Some("play.other.net"), false, false).is_none());
    assert!(s.frame(Some("play.other.net"), true, false).is_none());
    assert_eq!(
        s.frame(Some("play.other.net"), true, true),
        Some("other.net".into())
    );
    assert!(s.frame(None, false, false).is_none());
    assert!(s.frame(Some("play.other.net"), true, true).is_none());
}
