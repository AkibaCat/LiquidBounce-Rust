#!/usr/bin/env python3
"""BetterChat: actual chat storage, wrapped-line UI, packet receive and sendChat boundaries."""
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent

def apply(edit,once):
 for target,owned in [('crates/horizon-client/src/game/liquidbounce_better_chat.rs','better-chat-game.rs'),('crates/horizon-client-render/src/liquidbounce_better_chat.rs','better-chat-render.rs'),('crates/horizon-client-render/src/liquidbounce_better_chat_layout.rs','better-chat-layout.rs')]:
  edit(target,lambda _,owned=owned:(HERE/owned).read_text(encoding='utf8'))
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,'    pub(super) fn native_client_better_chat(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_better_chat(self,mod_api::mods(),input,output);}}\n'+anchor)
  return once(s,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_better_chat_tick();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def render(s):
  s=once(s,'    /// Local hour and minute it came in (PvP & HUD\'s chat timestamps).','    liquidbounce_id: Option<String>,\n    liquidbounce_count: u64,\n    /// Local hour and minute it came in (PvP & HUD\'s chat timestamps).')
  s=once(s,'pub struct Chat {','pub struct Chat {\n    liquidbounce_infinite: bool,\n    liquidbounce_anti_clear: bool,')
  s=once(s,'    last.signature.is_none()','    last.liquidbounce_id == next.liquidbounce_id\n        && last.signature.is_none()')
  s=once(s,'        self.messages.truncate(100);','        if !self.liquidbounce_infinite { self.messages.truncate(100); }')
  s=once(s,'        self.lines.truncate(100);','        if !self.liquidbounce_infinite { self.lines.truncate(100); }')
  s=once(s,'    pub fn clear(&mut self) {','    pub fn clear(&mut self) {\n        if self.liquidbounce_anti_clear { return; }')
  # FlagCheck also adds an inline metadata constructor. Match expressions,
  # excluding the Message struct declaration, so every accepted constructor
  # receives BetterChat metadata regardless of formatting.
  pattern=r'(?<!struct )\bMessage\s*\{'
  s,n=re.subn(pattern,lambda m:m.group(0)+' liquidbounce_id: None, liquidbounce_count: 1,',s)
  expected=4 if 'client_id: Option<String>' in s else 3
  if n!=expected:raise ValueError(f'Expected {expected} Chat Message constructors, got {n}')
  return s+'\ninclude!("liquidbounce_better_chat.rs");\n'
 edit('crates/horizon-client-render/src/chat.rs',render)
 def game(s):
  s=once(s,'mod native_hooks;','mod native_hooks;\nmod liquidbounce_better_chat;')
  s=once(s,'                        Screen::Chat if pressed && button == MouseButton::Left => self.chat_click(),','''                        Screen::Chat if pressed && matches!(button,MouseButton::Left|MouseButton::Right) => {
                            self.client_better_chat_copy(button);
                            if button==MouseButton::Left { self.chat_click(); }
                        },''')
  s=once(s,'                    if self.screen == Screen::Dead && event.state == ElementState::Pressed =>\n                {','''                    if self.screen == Screen::Dead && event.state == ElementState::Pressed =>
                {
                    if let PhysicalKey::Code(key)=event.physical_key {
                        if self.client_better_chat_death_key(key,event.text.as_deref()) { return Ok(()); }
                    }''')
  return s
 edit('crates/horizon-client/src/game.rs',game)
 def chat(s):
  s=once(s,'        self.ui.chat.draw(batch, &self.gui.atlas, now, focused);','        self.ui.chat.draw(batch, &self.gui.atlas, now, focused);\n        self.client_better_chat_highlight(batch);')
  s=once(s,'    pub(super) fn draw_chat(&mut self, batch: &mut HudBatch) {','    pub(super) fn draw_chat(&mut self, batch: &mut HudBatch) {\n        self.client_better_chat_tick();')
  s=once(s,'    fn local_chat_line(&mut self, text: Text, indicator: Option<u32>, source: ChatSource) {','''    fn local_chat_line(&mut self, text: Text, indicator: Option<u32>, source: ChatSource) {
        if source!=ChatSource::Client {
            let plain=text.plain();
            if !self.client_better_chat_receive(if source==ChatSource::Player{0}else{2},&plain,&text,source){return;}
        }''')
  # Single-player's direct chat takes the same sendChat modifier, after history and client commands.
  anchor='''        } else {
            let line = Text::translate(
                "chat.type.text",
                &[self.player_display_name(), Text::literal(message.as_str())],
            );
            let focused = self.screen == Screen::Chat;
            self.ui.chat.add(
                &self.gui.atlas,
                line,
                None,
                ChatSource::Player,
                self.visual_ticks(),
                focused,
            );
        }
    }
    pub(super) fn chat_scroll'''
  replacement='''        } else {
            let message=self.client_better_chat_outgoing(&message);
            let line = Text::translate("chat.type.text",&[self.player_display_name(),Text::literal(message)]);
            self.local_chat_line(line,None,ChatSource::Player);
        }
    }
    pub(super) fn chat_scroll'''
  s=once(s,anchor,replacement)
  # Inventory and keymapping extensions wrap the Playing transition before
  # BetterChat runs. Preserve that accepted expression and replace only the
  # close_chat branch whose trailing removed-doc anchor identifies it.
  close=re.compile(r'        if self\.screen == Screen::Chat \{\n            self\.screen = (?P<screen>[^\n]+);\n            self\.capture\(true\);\n        \}\n    \}\n    /// `ChatScreen\.removed`')
  matches=list(close.finditer(s))
  if len(matches)!=1:raise ValueError(f'Expected one close_chat transition, got {len(matches)}')
  match=matches[0]
  s=once(s,match.group(0),'''        if self.screen == Screen::Chat {
            let dead=self.player.as_ref().is_some_and(|p|p.health.dead_or_dying());
            let next_screen = '''+match.group('screen')+''';
            self.screen = if dead { Screen::Dead } else { next_screen };
            self.capture(!dead);
        }
    }
    /// `ChatScreen.removed`''')
  return s
 edit('crates/horizon-client/src/game/chat.rs',chat)
 def remote(s):
  s=once(s,'struct QueuedChat {','struct QueuedChat {\n    better_chat_plain: Option<String>,')
  # The ordinary player ChatReceiveEvent observes the already decorated component;
  # disguised events observe their original component before ChatType.decorate.
  s=once(s,'''        self.handle_player_chat(QueuedChat {
            line,
            signature: None,
            trust: ChatTrust::Error,''','''        self.handle_player_chat(QueuedChat {
            better_chat_plain: None,
            line,
            signature: None,
            trust: ChatTrust::Error,''')
  s=once(s,'''                self.handle_player_chat(QueuedChat {
                    line,
                    signature,
                    trust,''','''                self.handle_player_chat(QueuedChat {
                    better_chat_plain: None,
                    line,
                    signature,
                    trust,''')
  s=once(s,'''                self.handle_player_chat(QueuedChat {
                    line,
                    signature: None,
                    trust: ChatTrust::Secure,''','''                self.handle_player_chat(QueuedChat {
                    better_chat_plain: Some(horizon_client::component::to_text(&content,&mut |s|horizon_client::text::Text::literal(s)).plain()),
                    line,
                    signature: None,
                    trust: ChatTrust::Secure,''')
  s=once(s,'        let QueuedChat {\n            mut line,','        let QueuedChat {\n            better_chat_plain,\n            mut line,')
  s=once(s,'        let source = horizon_client::chat::ChatSource::Player;\n        let shown = self.signed_chat_line(line, source, signature, trust);','''        let source = horizon_client::chat::ChatSource::Player;
        if !error {
            let kind=if better_chat_plain.is_some(){1}else{0};
            let plain=better_chat_plain.unwrap_or_else(||line.plain());
            if !self.client_better_chat_receive(kind,&plain,&line,source){return false;}
        }
        let shown = self.signed_chat_line(line, source, signature, trust);''')
  s=once(s,'''                if !self.chat_received(true, p.overlay, 0, None, &mut text) {
                    return Ok(());
                }
                if p.overlay {''','''                if !self.chat_received(true, p.overlay, 0, None, &mut text) {
                    return Ok(());
                }
                let plain=text.plain();
                if !self.client_better_chat_receive(2,&plain,&text,horizon_client::chat::ChatSource::Server){return Ok(());}
                if p.overlay {''')
  s=once(s,'            self.send_signed_chat(message);','            let message=self.client_better_chat_outgoing(message);\n            self.send_signed_chat(&message);')
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/control.rs',lambda s:once(s,'    fn control_key_down(&mut self, key: KeyCode, raw: KeyCode) -> Result<()> {','    fn control_key_down(&mut self, key: KeyCode, raw: KeyCode) -> Result<()> {\n        if self.client_better_chat_death_key(raw,None){return Ok(());}') )
 return {'hook_names':['client_better_chat'],'helper_names':[]}
