#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};
type Events = Rc<RefCell<Vec<&'static str>>>;
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hit(u8);
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hand {
    Main,
    Off,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum GuestUse {
    OnBlock(Hand),
    Item(Hand),
}
#[derive(Clone)]
struct Stack {
    count: i32,
    item: Box<str>,
}
struct Heights(Events);
impl Heights {
    fn item_used(&mut self, _: Hand) {
        self.0.borrow_mut().push("itemUsed");
    }
}
struct Game<'a> {
    forced_hit: Option<Hit>,
    guest_use: Option<GuestUse>,
    client_placement_result: Option<i8>,
    remote: Option<()>,
    use_failed: bool,
    creative: bool,
    heights: Heights,
    stack: Option<Stack>,
    events: Events,
    block: (bool, i8),
    item: (bool, i8),
    consume: bool,
    error: bool,
    lifetime: std::marker::PhantomData<&'a ()>,
}
impl Game<'_> {
    fn with_hand<T>(&mut self, _: Hand, f: impl FnOnce(&mut Self) -> T) -> T {
        f(self)
    }
    fn held(&self) -> Option<&Stack> {
        self.stack.as_ref()
    }
    fn native_with_item_rotation<T>(
        &mut self,
        _: f32,
        _: f32,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        f(self)
    }
    fn remote_send_carried(&mut self) {
        self.events.borrow_mut().push("carried");
    }
    fn dispatch_block(&mut self) -> bool {
        self.events.borrow_mut().push("useOn");
        self.client_placement_result = Some(self.block.1);
        if self.consume {
            self.stack.as_mut().unwrap().count -= 1;
        }
        self.block.0
    }
    fn remote_use_item_on_at(&mut self, _: Hand, _: Option<Hit>) -> bool {
        self.dispatch_block()
    }
    fn use_hand(&mut self, _: Option<&Hit>) -> Result<bool, ()> {
        if self.error {
            Err(())
        } else {
            Ok(self.dispatch_block())
        }
    }
    fn dispatch_item(&mut self) -> bool {
        self.events.borrow_mut().push("useItem");
        self.client_placement_result = Some(self.item.1);
        self.item.0
    }
    fn remote_use_item_rotated(&mut self, _: Hand, _: f32, _: f32) -> bool {
        self.dispatch_item()
    }
    fn client_use_item_only(&mut self, _: bool) -> Result<bool, ()> {
        Ok(self.dispatch_item())
    }
    fn auto_rod_animation(&self, _: Hand) -> u8 {
        0
    }
    fn swing_hand(&mut self, _: Hand, _: u8) {
        self.events.borrow_mut().push("swing");
    }
}
#[path = "placement-use-native.rs"]
mod placement_use;
fn fixture(remote: bool) -> Game<'static> {
    let events = Events::default();
    Game {
        forced_hit: Some(Hit(7)),
        guest_use: Some(GuestUse::Item(Hand::Off)),
        client_placement_result: Some(7),
        remote: remote.then_some(()),
        use_failed: false,
        creative: false,
        heights: Heights(events.clone()),
        stack: Some(Stack {
            count: 2,
            item: "minecraft:water_bucket".into(),
        }),
        events,
        block: (true, 1),
        item: (true, 1),
        consume: true,
        error: false,
        lifetime: std::marker::PhantomData,
    }
}
fn execute(g: &mut Game<'_>, swing_mode: &str, callback_result: bool) -> bool {
    g.client_do_placement(Hit(1), [35., 80.], Hand::Off, swing_mode, |g, item| {
        g.events.borrow_mut().push(if item {
            "itemCallback"
        } else {
            "placementCallback"
        });
        callback_result
    })
}
fn restored(g: &Game<'_>) {
    assert_eq!(g.forced_hit, Some(Hit(7)));
    assert_eq!(g.guest_use, Some(GuestUse::Item(Hand::Off)));
    assert_eq!(g.client_placement_result, Some(7));
}
fn main() {
    for remote in [false, true] {
        let mut g = fixture(remote);
        assert!(execute(&mut g, "DoNotHide", true));
        restored(&g);
        assert_eq!(
            *g.events.borrow(),
            ["carried", "useOn", "placementCallback", "swing", "itemUsed"]
        );
        for hidden in ["HideForClient", "HideForBoth"] {
            let mut g = fixture(remote);
            assert!(execute(&mut g, hidden, true));
            assert_eq!(
                *g.events.borrow(),
                ["carried", "useOn", "placementCallback", "itemUsed"]
            );
        }
        let mut g = fixture(remote);
        assert!(execute(&mut g, "HideForServer", false));
        assert_eq!(
            *g.events.borrow(),
            ["carried", "useOn", "placementCallback", "itemUsed"]
        );
        // CONSUME in useOn invokes onPlacementSuccess even without a predicted swing.
        let mut g = fixture(remote);
        g.block.1 = 0;
        assert!(execute(&mut g, "DoNotHide", true));
        assert_eq!(
            *g.events.borrow(),
            ["carried", "useOn", "placementCallback", "swing", "itemUsed"]
        );
        for code in [-1, -2] {
            let mut g = fixture(remote);
            g.block.1 = code;
            assert!(!execute(&mut g, "DoNotHide", true));
            restored(&g);
            assert_eq!(*g.events.borrow(), ["carried", "useOn"]);
        }
        let mut g = fixture(remote);
        g.block = (false, 0);
        g.consume = false;
        assert!(execute(&mut g, "DoNotHide", true));
        restored(&g);
        assert_eq!(
            *g.events.borrow(),
            [
                "carried",
                "useOn",
                "useItem",
                "itemCallback",
                "swing",
                "itemUsed"
            ]
        );
        for item in [(false, 0), (true, 0), (true, -2)] {
            let mut g = fixture(remote);
            g.block = (false, 0);
            g.item = item;
            g.consume = false;
            assert!(!execute(&mut g, "DoNotHide", true));
            assert_eq!(*g.events.borrow(), ["carried", "useOn", "useItem"]);
        }
        let mut g = fixture(remote);
        g.block = (false, 0);
        g.stack = None;
        g.consume = false;
        assert!(!execute(&mut g, "DoNotHide", true));
        assert_eq!(*g.events.borrow(), ["carried", "useOn"]);
        let mut g = fixture(remote);
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g.client_do_placement(Hit(1), [0.; 2], Hand::Main, "DoNotHide", |_, _| {
                panic!("callback fixture")
            })
        }));
        assert!(panicked.is_err());
        restored(&g);
    }
    let mut g = fixture(false);
    g.error = true;
    assert!(!execute(&mut g, "DoNotHide", true));
    restored(&g);
    println!(
        "doPlacement real helper: callback ordering, PASS/FAIL/CONSUME/Success, hiding, both hands and panic restoration passed"
    );
}
