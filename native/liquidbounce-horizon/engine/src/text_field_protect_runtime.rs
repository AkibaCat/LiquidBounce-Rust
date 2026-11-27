// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{RenderHooks, TextFieldFormatEvent};

impl RenderHooks for LiquidBounceHooks {
    fn text_field_format(&self, event: &mut TextFieldFormatEvent<'_>) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        let running = modules.query_running(&state.registry, "TextFieldProtect");
        let result =
            modules
                .text_field_protect
                .protect(running, event.input, event.first_character_index);
        let masked = match result {
            Ok(std::borrow::Cow::Borrowed(_)) => false,
            Ok(std::borrow::Cow::Owned(value)) => {
                *event.input = value;
                true
            }
            Err(error) => panic!("TextFieldProtect EditBox.applyFormat: {error:?}"),
        };
        drop(modules);
        #[cfg(feature = "webview")]
        {
            let mut state = state;
            // Integration evidence needs counts, never the field's contents.
            state.probe("textFieldFormat",
                serde_json::json!({"running":running,"firstCharacterIndex":event.first_character_index}),
                serde_json::json!({"masked":masked,"renderedLength":event.input.encode_utf16().count()}));
        }
        #[cfg(not(feature = "webview"))]
        let _ = masked;
    }
}

#[cfg(all(test, feature = "webview"))]
mod tests {
    use super::*;
    use crate::{actor::Actor, behaviors::RawOptions};
    use horizon_client::{
        gfx::native_render,
        hud::{HudAtlas, HudBatch},
        mod_api::Hooks,
        screen_draw::{Measure, TextField},
        text::Text,
        ui,
    };
    use liquidbounce_core::WorldState;
    use liquidbounce_interop::protocol::ModuleAccess;
    use serde_json::json;
    use std::cell::RefCell;

    #[test]
    fn actual_registry_render_dispatch_and_editbox_formatter_arguments() {
        let hooks = Box::leak(Box::new(LiquidBounceHooks::default()));
        let options = RawOptions {
            fov: 70,
            gamma: 0.5,
            player_rotation: [0.0; 2],
            actual_main_hand: 1,
            model_parts: 127,
        };
        {
            let mut state = hooks.state();
            state.with_registry(options, |registry, callbacks| {
                registry.update_world_sync(
                    WorldState {
                        world_present: true,
                        player_present: true,
                    },
                    callbacks,
                )
            });
        }
        assert!(
            native_render::install(
                vec![Hooks::render(hooks).expect("actual render adapter")].into_boxed_slice()
            )
            .is_ok()
        );
        let atlas = HudAtlas::build(std::path::Path::new(
            "/nonexistent/text-field-protect-assets",
        ))
        .unwrap();
        let measure = Measure(&atlas.font);
        let arguments = RefCell::new(Vec::<(String, usize)>::new());
        let formatter = |input: &str, offset: usize| {
            arguments.borrow_mut().push((input.to_owned(), offset));
            Text::literal(input)
        };
        let mut field = TextField::new(0, 0, 300, 20);
        let mut draws = Vec::new();
        let mut draw = |value: &str, display: usize, cursor: usize| {
            field.value = value.to_owned();
            field.display = display;
            field.cursor = cursor;
            field.highlight = cursor;
            arguments.borrow_mut().clear();
            draws.clear();
            field.draw(&mut draws, &measure, 0, Some(&formatter));
            assert_eq!(field.value, value);
            arguments.borrow().clone()
        };
        assert_eq!(
            draw("/login secret", 0, 13),
            [("/login secret".to_owned(), 0)]
        );
        {
            let mut state = hooks.state();
            let mut actor = Actor {
                state: &mut state,
                options,
                entity_interaction_attribute: None,
            };
            actor.set_enabled("TextFieldProtect", true).unwrap();
        }
        // Unfocused boxes use the same hook; displayPos zero masks to empty.
        assert_eq!(draw("/login secret", 0, 13), [(String::new(), 0)]);
        // Both applyFormat calls test their own substring, not the whole input.
        assert_eq!(
            draw("/login secret", 0, 4),
            [("/log".to_owned(), 0), ("in secret".to_owned(), 4)]
        );
        // The emoji preceding the visible text contributes two UTF-16 units.
        assert_eq!(draw("😀/login secret", 1, 14), [("**".to_owned(), 1)]);
        // The second call uses displayPos again, independently of its offset.
        assert_eq!(
            draw("😀abc/login secret", 1, 4),
            [("abc".to_owned(), 1), ("**".to_owned(), 4)]
        );

        let mut edit = ui::EditBox::new(0.0, 0.0, 300.0, 20.0);
        edit.bordered = false;
        edit.focused = false;
        edit.set_value("/login secret");
        let mut batch = HudBatch::new([854, 480]);
        edit.draw(&mut batch, &atlas, None);
        assert!(batch.vertices.is_empty());
        assert_eq!(edit.value, "/login secret");
        edit.multiline = true;
        edit.draw(&mut batch, &atlas, None);
        assert!(
            !batch.vertices.is_empty(),
            "MultiLineEditBox is outside the original mixin"
        );
        let mut append = HudBatch::new([854, 480]);
        ui::append_field(
            &mut append,
            &atlas,
            "/login secret",
            (0.0, 0.0, 300.0),
            (u32::MAX, false),
            100,
            false,
            false,
            300,
        );
        assert!(append.vertices.is_empty());
        let styled = Text::literal("a")
            .with_color(0xAA0000)
            .append(Text::literal("bc").with_color(0x00AA00));
        let formatted = ui::format_field_argument(Some(&styled), "**", 1);
        assert_eq!(formatted.plain(), "**");
        assert!(
            formatted
                .spans
                .iter()
                .all(|span| span.style.color == Some(0x00AA00))
        );
        {
            let mut state = hooks.state();
            let mut actor = Actor {
                state: &mut state,
                options,
                entity_interaction_attribute: None,
            };
            actor.set_settings("TextFieldProtect", &json!({"name":"TextFieldProtect", "value":[{"name":"Patterns", "value":["^ordinary$"]}]})).unwrap();
        }
        assert_eq!(
            draw("/login secret", 0, 13),
            [("/login secret".to_owned(), 0)]
        );
        assert_eq!(draw("ordinary", 0, 8), [(String::new(), 0)]);
        {
            let mut state = hooks.state();
            let mut actor = Actor {
                state: &mut state,
                options,
                entity_interaction_attribute: None,
            };
            actor.set_enabled("TextFieldProtect", false).unwrap();
        }
        assert_eq!(draw("ordinary", 0, 8), [("ordinary".to_owned(), 0)]);
    }
}
