"""Native Font draw/width hooks and local style bypass; line splitting keeps original text."""
from pathlib import Path
HERE=Path(__file__).resolve().parent


def apply(edit,once):
    def text(s):
        s=once(s,'    pub color: Option<u32>,','    pub color: Option<u32>,\n    /// Local TextColorAddition flag; tied to the explicit/inherited color, never serialized.\n    pub bypasses_name_protection: bool,')
        s=once(s,'        self.color = self.color.or(base.color);','        if self.color.is_none(){self.bypasses_name_protection=base.bypasses_name_protection;}\n        self.color = self.color.or(base.color);')
        s=once(s,'            color: Some(c.2),','            color: Some(c.2),\n            bypasses_name_protection:false,')
        s=once(s,'    pub fn colored(s: impl Into<String>, color: u32) -> Self {','''    /// ClientChat.bypassNameProtection: the color carries the bypass through styled sequences.
    pub fn bypass_name_protection(mut self)->Self {
        self.source=None;
        for span in &mut self.spans {span.style.color.get_or_insert(WHITE);span.style.bypasses_name_protection=true;}
        self
    }
    pub fn colored(s: impl Into<String>, color: u32) -> Self {''')
        return s+'\n'+(HERE/'name-protect-text-service.rs').read_text(encoding='utf8')
    edit('crates/horizon-client-core/src/text.rs',text)

    def font(s):
        old='''    pub fn width(&self, s: &str) -> f32 {
        if s.contains('§') {
            return self.width_text(&Text::legacy(s));
        }
        crate::bidi::visual_str(s)
            .chars()
            .map(|c| self.glyph(c).advance)
            .sum::<f32>()
            .ceil()
    }'''
        new='''    pub fn width(&self, s: &str) -> f32 {
        // String overload replaces before legacy parsing, once. Sequence overload is separate.
        let protected=crate::text::protect_string(s);let s=protected.as_ref();
        if s.contains('§') {
            return crate::bidi::visual(&Text::legacy(s)).chars().map(|(c,s)|self.advance(c,s)).sum::<f32>().ceil();
        }
        crate::bidi::visual_str(s).chars().map(|c|self.glyph(c).advance).sum::<f32>().ceil()
    }'''
        s=once(s,old,new)
        s=once(s,'    pub fn width_text(&self, t: &Text) -> f32 {\n        let t = t.resolved();','    pub fn width_text(&self, t: &Text) -> f32 {\n        let t = t.resolved();\n        let t = crate::text::protect_text(&t);')
        return once(s,'    pub fn width_drawn(&self, t: &Text) -> f32 {\n        crate::bidi::visual(t)','    pub fn width_drawn(&self, t: &Text) -> f32 {\n        let t=crate::text::protect_text(t);\n        crate::bidi::visual(&t)')
    edit('crates/horizon-client-render/src/font.rs',font)

    def hud(s):
        s=once(s,"        let t = if text.contains('§') {","        let protected=crate::text::protect_string(text);let text=protected.as_ref();\n        let t = if text.contains('§') {")
        s=once(s,'        self.text_styled(atlas, &t, x, y, color, shadow)','        self.styled_text(atlas, &t, x, y, color, shadow, false, None, false)')
        s=once(s,'        self.styled_text(atlas, text, x, y, color, shadow, false, None)','        self.styled_text(atlas, text, x, y, color, shadow, false, None, true)')
        s=once(s,'        self.styled_text(atlas, text, x, y, color, shadow, true, None)','        self.styled_text(atlas, text, x, y, color, shadow, true, None, true)')
        s=once(s,'                        Some((outline, [dx as f32, dy as f32])),','                        Some((outline, [dx as f32, dy as f32])),\n                        true,')
        s=once(s,'            self.styled_text(atlas, text, x, y, color, false, true, None),','            self.styled_text(atlas, text, x, y, color, false, true, None, true),')
        s=once(s,'        outline: Option<(u32, [f32; 2])>,\n    ) -> f32 {','        outline: Option<(u32, [f32; 2])>,\n        protect_names:bool,\n    ) -> f32 {')
        return once(s,'        let text = text.resolved();\n        // Language.getVisualOrder:', '        let text = text.resolved();\n        let text=if protect_names{crate::text::protect_text(&text)}else{std::borrow::Cow::Borrowed(text.as_ref())};\n        // Language.getVisualOrder:')
    edit('crates/horizon-client-render/src/hud.rs',hud)

    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'name-protect-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
    return {'hook_names':[],'helper_names':['client_name_protect_snapshot','client_name_protect_friend']}
