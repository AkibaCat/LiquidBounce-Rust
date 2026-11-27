    pub(super) fn native_client_gui_closer_screen(&self, title: &str, cancel: &mut bool, print_title: &mut bool) {
        mod_api::dispatch_client_gui_closer_screen(self, mod_api::mods(), title, cancel, print_title);
    }
    /// ScreenEvent is delivered at setScreen HEAD, before replacing/removing the old screen.
    pub(super) fn client_gui_closer_accept_screen(&mut self, next: super::Screen) -> bool {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return self.client_gui_closer_finish_screen(true); }
        let Some((_, title)) = self.client_container_menu_title(next) else { return self.client_gui_closer_finish_screen(true); };
        let (mut cancel, mut print_title) = (false, false);
        self.native_client_gui_closer_screen(&title, &mut cancel, &mut print_title);
        if print_title { self.system_message(Self::client_gui_closer_title_message(&title)); }
        self.client_gui_closer_finish_screen(!cancel)
    }
    fn client_gui_closer_title_message(title: &str) -> horizon_client::text::Text {
        use horizon_client::text::{ClickEvent, Style, Text};
        let mut text = Text::empty();
        // ClientChat.clientTag uses Java float division and Color4b's fma/toInt.
        for (index, character) in "LiquidBounce".chars().enumerate() {
            let factor = index as f32 / 11.0;
            let channel = |first: f32, last: f32| factor.mul_add(last - first, first) as u32;
            let color = (channel(70.0, 36.0) << 16) | (channel(119.0, 170.0) << 8) | channel(255.0, 127.0);
            text = text.append(Text::colored(character.to_string(), color));
        }
        // The user's Chinese-display requirement localizes the hardcoded
        // English body; the tag, title text and interactive style stay exact.
        let language = horizon_client::lang::current();
        let body = match language.as_ref().map(|v| v.code.as_str()) {
            Some("zh_cn") => " ▸ GUICloser: 容器界面标题: ",
            Some("zh_tw") => " ▸ GUICloser: 容器介面標題: ",
            _ => " ▸ GUICloser: container screen title: ",
        };
        text.append(Text::colored(body, 0xaaaaaa))
            .append(Text::styled(title, Style {
                color: Some(0x4677ff), bold: Some(true),
                click: Some(ClickEvent::CopyToClipboard(title.to_owned())),
                hover: Some(Box::new(Text::translate("chat.copy.click", &[]))),
                ..Style::default()
            }))
    }
