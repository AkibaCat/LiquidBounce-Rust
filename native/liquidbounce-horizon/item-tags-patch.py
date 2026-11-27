"""Native ItemTags tick snapshots, live entities, model items and original panel SDF."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    edit('crates/horizon-client/src/game/item_tags.rs',lambda _:(HERE/'item-tags-game.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/item_stack_list.rs',lambda _:(HERE/'item-stack-list-game.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game/item_tags_layout.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/item_tags_layout.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client-render/src/item-tags-hud.rs',lambda _:(HERE/'item-tags-hud.rs').read_text(encoding='utf8'))
    def game(s):
        s=once(s,'mod radar;','mod radar;\nmod item_tags;\nmod item_stack_list;')
        s=once(s,'    native_mod_screen: Option<String>,','    native_mod_screen: Option<String>,\n    client_item_tags:std::cell::RefCell<item_tags::State>,\n    client_item_tags_alpha:std::cell::Cell<f64>,\n    client_item_stack_lists:std::cell::RefCell<item_stack_list::Queue>,')
        s=once(s,'            native_mod_screen: None,','            native_mod_screen: None,\n            client_item_tags:Default::default(),\n            client_item_tags_alpha:std::cell::Cell::new(0.),\n            client_item_stack_lists:Default::default(),')
        s=once(s,'        let mut hud = horizon_client::spans::time("hud", || self.hud(size));','        self.client_item_stack_lists.borrow_mut().0.clear();\n        self.client_item_tags_alpha.set(alpha);\n        let mut hud = horizon_client::spans::time("hud", || self.hud(size));')
        s=once(s,'            self.client_tnt_timer_submit(&mut batch);','            self.client_item_tags_overlay(&mut batch);\n            self.client_tnt_timer_submit(&mut batch);')
        return once(s,'        // Hud.extractSleepOverlay: over all health/food bars, under titles and chat.','        self.client_item_stack_lists_flush(&mut batch);\n        // Hud.extractSleepOverlay: over all health/food bars, under titles and chat.')
    edit('crates/horizon-client/src/game.rs',game)
    def hooks(s):
        a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        s=once(s,a,'    pub(super) fn native_client_item_tags(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_item_tags(self,mod_api::mods(),input,output);}}\n\n'+a)
        s=once(s,'        self.client_radar_tick();','        self.client_radar_tick();\n        self.client_item_tags_tick();')
        return once(s,'        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);','        *self.client_item_tags.borrow_mut()=Default::default();\n        self.client_item_stack_lists.borrow_mut().0.clear();\n        mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    def chest(s):
        start=s.index('        let title_height = if title.is_some() { 11. } else { 0. };')
        panel=s.index('    fn chest_stealer_panel(',start)
        end=s.index('{',panel)+1;depth=1
        while depth:depth+=(s[end]=='{')-(s[end]=='}');end+=1
        # The source overload defaults to rearrange=false. Keep that behavior;
        # lists requesting rearrange share the same queue as ItemTags/BedPlates.
        return s[:start]+'''        let scale=options["scale"].as_f64().unwrap_or(1.5)as f32;
        self.client_item_stack_list(batch,super::item_stack_list::List{
            stacks:slots,title:title.map(horizon_client::text::Text::literal),title_color:u32::MAX,
            row_length:9,center:[cx,cy],scale,background:options,renderer:None,
        },false);
    }
'''+s[end:]
    edit('crates/horizon-client/src/game/chest_stealer.rs',chest)
    def hud(s):
        s=once(s,'wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Unorm8x4,3=>Float32,4=>Float32x2]','wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Unorm8x4,3=>Float32,4=>Float32x2,5=>Uint32]')
        return s+'\ninclude!("item-tags-hud.rs");\n'
    edit('crates/horizon-client-render/src/hud.rs',hud)
    edit('crates/horizon-client-render/src/hud/pack_text.rs',lambda s:once(s,'|v| v.text != 0','|v| v.text == 1'))
    def shader(s):
        s=once(s,'    @location(3) glint_uv: vec2<f32>,','    @location(3) glint_uv: vec2<f32>,\n    @location(4) @interpolate(flat) shape:u32,')
        s=once(s,'@location(4) glint_uv: vec2<f32>) -> Out','@location(4) glint_uv: vec2<f32>,@location(5) shape:u32) -> Out')
        s=once(s,'    out.glint_uv = glint_uv;','    out.glint_uv = glint_uv;\n    out.shape=shape;')
        s=once(s,'@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {','@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {\n    if (in.shape&0x80000000u)!=0u{let rounded=liquidbounce_rounded(in);if rounded.a<=0.001{discard;}return rounded;}')
        return s+'\n'+(HERE/'item-tags-shader.wgsl').read_text(encoding='utf8')
    edit('crates/horizon-client-render/src/hud.wgsl',shader)
    return {'hook_names':['client_item_tags'],'helper_names':[]}
