// The real generator's event/dispatch/Host outputs compile at their real module boundaries.
extern crate self as horizon_client;

mod mod_api {
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _name: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|module| &**module)
    }
    mod render {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"));
    }
    pub use render::*;
    mod host { include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs")); }
    pub use host::*;
    mod hooks { include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs")); }
    pub use hooks::*;
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::OnceLock<Vec<Box<dyn Hooks>>> = std::sync::OnceLock::new();
        MODS.get_or_init(|| vec![Box::new(TestHooks)])
    }
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_camera_aspect(&self, _host: &dyn Host, event: &mut ClientCameraAspectEvent) {
            event.ratio_percentage = 200;
        }
        fn client_camera_position(&self, host: &dyn Host, event: &mut ClientCameraPositionEvent<'_>) {
            assert_eq!(host.raw_camera_perspective(), 1);
            assert_eq!(event.raw_perspective, 1);
            event.position[0] += 5.;
            event.position[1] *= 2.;
            event.position[2] = -3.;
        }
        fn client_camera_perspective(&self, _host: &dyn Host, event: &mut ClientCameraPerspectiveEvent) {
            assert_eq!(event.original, 1);
            if !event.managed_camera { event.perspective = 2; }
        }
    }
}

mod game {
    pub mod perspective {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum CameraType { FirstPerson, ThirdPersonBack, ThirdPersonFront }
    }
    struct Studio(bool);
    impl Studio { fn flying(&self) -> bool { self.0 } }
    struct Game<'a> {
        camera_type: perspective::CameraType,
        studio: Studio,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> { fn native_window(&self) -> bool { true } }
    mod native_hooks { use super::Game; use crate::mod_api; __NATIVE_HOOK_METHODS__ }
    mod mod_api_host { include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs")); }
    #[test]
    fn real_generator_preserves_mutable_array_events_and_host_helper_namespace() {
        use crate::mod_api::Host;
        let mut game=Game{camera_type:perspective::CameraType::ThirdPersonBack,
            studio:Studio(false),marker:std::marker::PhantomData};
        // These calls cross from game/native_hooks.rs to its actual parent;
        // a private sync/helper/effective method must fail compilation here.
        game.sync_native_camera_aspect();
        assert_eq!(crate::camera_control::aspect(1920.,1080.),1920./1080./2.);
        assert_eq!(Host::raw_camera_perspective(&game),1);
        let mut position=[10.,20.,30.];
        game.native_client_camera_position(1,&mut position);
        assert_eq!(position,[15.,40.,-3.]);
        assert_eq!(game.effective_camera_type(),perspective::CameraType::ThirdPersonFront);
        assert_eq!(game.camera_type,perspective::CameraType::ThirdPersonBack);
        game.studio.0=true;
        assert_eq!(game.effective_camera_type(),perspective::CameraType::ThirdPersonBack);
        crate::camera_control::publish(100);
    }
}
