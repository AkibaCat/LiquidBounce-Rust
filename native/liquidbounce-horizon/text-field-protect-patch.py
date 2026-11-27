#!/usr/bin/env python3
"""EditBox formatter argument hook in the lower rendering crate, without gameplay dependencies."""
import json


def apply(edit, once):
    def generator(text):
        # fill_hooks rustfmts committed code. Multi-argument dispatch functions
        # gain trailing commas, which syn preserves in its token stream. Compare
        # against the same formatted candidate, retaining the stale-code check.
        text = once(text, '''    if tokens(&committed)? == tokens(render)? {''', '''    let formatted = rustfmt(render)?;
    let formatted = std::str::from_utf8(&formatted).map_err(|e| e.to_string())?;
    if tokens(&committed)? == tokens(formatted)? {''')
        return text + '''
#[cfg(test)]
mod liquidbounce_render_generation_tests {
    #[test]
    fn formatted_multi_argument_render_dispatch_is_current_and_changes_fail() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!("liquidbounce-render-hooks-{}", std::process::id()));
        let path = root.join(super::RENDER_HOOKS);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let code = "pub trait RenderHooks { fn field(&self, event: &mut FieldEvent<'_>) {} }\\n\\
            pub struct FieldEvent<'a> { pub input: &'a mut String, pub first_character_index: i32, }\\n\\
            pub fn dispatch_field(mods: &[&dyn RenderHooks], input: &mut String, first_character_index: i32) {\\n\\
                let mut event = FieldEvent { input, first_character_index };\\n\\
                for hook in mods { hook.field(&mut event); }\\n\\
            }\\n";
        std::fs::write(&path, super::rustfmt(code).unwrap()).unwrap();
        super::render_hooks_current(&root, code).unwrap();
        let changed = code.replace("first_character_index: i32", "first_character_index: i64");
        assert!(super::render_hooks_current(&root, &changed).unwrap_err()
            .contains("is out of date with the render hooks"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
'''
    edit('crates/horizon-mod-api-gen/src/lib.rs', generator)
    def render(text):
        text = once(text, '    HandDrawnEvent, RenderHooks, RenderNeedsEvent, WorldDrawnEvent, WorldWarpEvent,',
                    '    HandDrawnEvent, RenderHooks, RenderNeedsEvent, WorldDrawnEvent, WorldWarpEvent, TextFieldFormatEvent,')
        return text + '''
/// EditBox.applyFormat's string argument, on each visible substring before
/// formatting. first_character_index is the field's displayPos in UTF-16 units.
/// This render hook also runs for unfocused fields; the actual input is untouched.
#[inline(never)]
pub fn native_text_field_format(input: &mut String, first_character_index: i32) {
    generated::dispatch_text_field_format(RENDERERS.get().map_or(&[], |renderers| renderers), input, first_character_index);
}
'''
    edit('crates/horizon-client-render/src/gfx/native_render.rs', render)
    edit('crates/horizon-client/src/mod_api.rs', lambda text: once(
        text, '    BlockSpriteArrays, RenderHooks, RenderNeeds, RenderNeedsEvent, WorldDrawn, WorldDrawnEvent,',
        '    BlockSpriteArrays, RenderHooks, RenderNeeds, RenderNeedsEvent, WorldDrawn, WorldDrawnEvent, TextFieldFormatEvent,'))
    def draw(text):
        return once(text, '''        let styled = |s: String, offset: usize| match format {
            Some(f) => f(&s, offset),
            None => Text::literal(s),
        };''', '''        let first_character_index = self.value.chars().take(self.display).map(char::len_utf16).sum::<usize>() as i32;
        let styled = |mut s: String, offset: usize| {
            crate::gfx::native_render::native_text_field_format(&mut s, first_character_index);
            match format {
                Some(f) => f(&s, offset),
                None => Text::literal(s),
            }
        };''')
    edit('crates/horizon-client-render/src/screen_draw.rs', draw)
    def ui(text):
        text = once(text, '''    let mut cursor_x = x;
    if !value.is_empty() {
        batch.text(atlas, value, x, y, color, shadow);
        cursor_x += HudBatch::text_width(atlas, value) + 1.0;
    }''', '''    let mut cursor_x = x;
    if !value.is_empty() {
        let mut rendered = value.to_owned();
        crate::gfx::native_render::native_text_field_format(&mut rendered, 0);
        batch.text(atlas, &rendered, x, y, color, shadow);
        cursor_x += HudBatch::text_width(atlas, &rendered) + 1.0;
    }''')
        text = once(text, '''        let styled = |from: usize, to: usize| -> Text {
            match format {
                Some(t) => t.slice(self.display + from, self.display + to),
                None => Text::literal(
                    displayed
                        .chars()
                        .skip(from)
                        .take(to - from)
                        .collect::<String>(),
                ),
            }
        };''', '''        let first_character_index = self.value.chars().take(self.display).map(char::len_utf16).sum::<usize>() as i32;
        let styled = |from: usize, to: usize| -> Text {
            let argument = displayed.chars().skip(from).take(to - from).collect::<String>();
            let mut rendered = argument.clone();
            if !self.multiline {
                crate::gfx::native_render::native_text_field_format(&mut rendered, first_character_index);
            }
            if rendered == argument {
                return match format {
                    Some(t) => t.slice(self.display + from, self.display + to),
                    None => Text::literal(rendered),
                };
            }
            format_field_argument(format, &rendered, self.display + from)
        };''')
        return text + '''
/// Apply the precomputed style ranges to the actual formatter argument. Most
/// EditBoxes use the literal formatter; chat uses screen_draw's live closure.
/// The style positions stay tied to the original field, while its string can
/// change in a render hook, just as EditBox.applyFormat(String, int).
pub fn format_field_argument(format: Option<&Text>, input: &str, offset: usize) -> Text {
    let Some(format) = format else { return Text::literal(input); };
    let styles = format.spans.iter().flat_map(|span| span.text.chars().map(move |_| &span.style));
    let mut styles = styles.skip(offset);
    let mut out = Text::empty();
    for ch in input.chars() {
        out.push(match styles.next() {
            Some(style) => Text::styled(ch.to_string(), style.clone()),
            None => Text::literal(ch.to_string()),
        });
    }
    out
}
'''
    edit('crates/horizon-client-render/src/ui.rs', ui)
    fragment = {'hooks': [{
        'name': 'text_field_format', 'version': 1,
        'target': 'horizon_client_render::gfx::native_render::native_text_field_format',
        'side': 'client', 'mod_api': {'contract': 'listen'},
    }], 'helpers': [], 'doc_summary': 'Actual EditBox.applyFormat string argument; UTF-16 displayPos, before each formatter invocation.'}
    edit('tools/native-hook-fragments/liquidbounce_text_field.json', lambda text: json.dumps(fragment, indent=2) + '\n')
    return {'hook_names': [], 'helper_names': []}
