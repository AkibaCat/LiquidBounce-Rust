    pub(super) fn native_client_name_protect_snapshot(&self)->String {
        serde_json::json!({"user":self.own_profile_name(),"online":self.client_anti_bot_online()}).to_string()
    }
    pub(super) fn native_client_name_protect_friend(&mut self,name:&str,alias:&str,prefix:&str) {
        use horizon_client::text::{Text,Style,ClickEvent,GOLD,GRAY,RED};
        let real_name=Text::styled(name,Style{italic:Some(true),click:Some(ClickEvent::CopyToClipboard(name.into())),hover:Some(Box::new(Text::translate("chat.copy.click",&[]))),..Style::color(GOLD)}).bypass_name_protection();
        let remove=match horizon_client::lang::code().to_ascii_lowercase().as_str(){"zh_cn"|"zh_sg"|"zh_hans"=>format!("移除 {name}"),"zh_tw"|"zh_hk"|"zh_mo"|"zh_hant"=>format!("移除 {name}"),_=>format!("Remove {name}")};
        let button=Text::styled("[X]",Style{bold:Some(true),click:Some(ClickEvent::SuggestCommand(format!("{prefix}friend remove {name}"))),hover:Some(Box::new(Text::colored(remove,GRAY))),..Style::color(RED)});
        let text=Text::colored("- ",GRAY).append(real_name).append(Text::colored(" (",GRAY)).append(Text::colored(alias,GOLD)).append(Text::colored(") ",GRAY)).append(button);
        self.chat_line(text,horizon_client::chat::ChatSource::Client);
    }
