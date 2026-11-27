from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client-render/src/better_tab.rs',lambda _:(HERE/'better-tab-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/lib.rs',lambda t:once(t,'pub mod tab_list;','pub mod tab_list;\npub mod better_tab;'))
 # render crate modules are re-exported through horizon-client.
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'    split_layout, split_map, split_profiles, sprites, streaming, structure_block, tab_list,','    better_tab, split_layout, split_map, split_profiles, sprites, streaming, structure_block, tab_list,'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'better-tab-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/scoreboard.rs',lambda t:once(t,'        if changed {\n            // PlayerTabOverlay.setVisible: the names, said as the list comes up.','        let _better_tab_scope=self.client_better_tab_scope(&players);\n        if changed {\n            // PlayerTabOverlay.setVisible: the names, said as the list comes up.'))
 def tab(t):
  t=once(t,'    out.truncate(80);\n    out.into_iter().map(|(p, _)| p).collect()','    crate::better_tab::ordered(players,out.into_iter().map(|(p, _)| p).collect())')
  t=once(t,'    order: Option<(Vec<TabPlayer>, Scoreboard, Vec<u128>)>,','    order: Option<(Vec<TabPlayer>, Scoreboard, Vec<u128>)>,\n    better_tab: Option<crate::better_tab::Policy>,')
  t=once(t,'impl TabList {\n','impl TabList {\n    fn sync_better_tab(&mut self){let current=crate::better_tab::current();if self.better_tab!=current{self.better_tab=current;self.cache=None;self.order=None;}}\n')
  t=once(t,'    pub fn displayed_ids(&mut self, players: &[TabPlayer], board: &Scoreboard) -> &[u128] {','    pub fn displayed_ids(&mut self, players: &[TabPlayer], board: &Scoreboard) -> &[u128] {\n        self.sync_better_tab();')
  # Sorting by ping/display length has additional cache inputs, even without a policy edit.
  t=once(t,'                        || a.game_mode != b.game_mode','                        || a.game_mode != b.game_mode\n                        || (self.better_tab.is_some() && (a.latency!=b.latency || a.display_name!=b.display_name))')
  t=once(t,'    pub fn layout(&mut self, i: &TabInputs, font: &Font) -> Vec<GuiDraw> {','    pub fn layout(&mut self, i: &TabInputs, font: &Font) -> Vec<GuiDraw> {\n        self.sync_better_tab();')
  t=once(t,'            let name = name_for_display(p, i.board).resolve_legacy();','            let name = crate::better_tab::name(p,name_for_display(p, i.board)).resolve_legacy();')
  t=once(t,'        let show_head = i.online_mode;','        let (cols,rows)=crate::better_tab::dimensions(slots,(cols,rows));\n        let show_head = i.online_mode;')
  t=once(t,'        let slot_width = (cols * (head + max_name + width_for_score + 13)).min(sw - 50) / cols;','        let slot_width = (cols * (head + max_name + width_for_score + 13)+crate::better_tab::width_extra()).min(sw - 50) / cols;')
  t=once(t,'        let header = split(&self.header);\n        let footer = split(&self.footer);','        let header = if crate::better_tab::visible(1){split(&self.header)}else{None};\n        let footer = if crate::better_tab::visible(2){split(&self.footer)}else{None};')
  t=once(t,'                color: i.background,','                color: crate::better_tab::color(p.uuid,i.background),')
  old='''            out.push(sprite(
                ping_sprite(p.latency),
                xo - head + slot_width - 11,
                yo,
                10,
                8,
            ));'''
  new='''            if let Some((text,color))=crate::better_tab::latency(p.latency){let text_width=width(&text);out.push(GuiDraw::Text{text,x:xo-head+slot_width-text_width,y:yo,color,shadow:false});}else{
'''+old+'''
            }'''
  return once(t,old,new)
 edit('crates/horizon-client-render/src/tab_list.rs',tab)
 return {'hook_names':['client_better_tab'],'helper_names':[]}
