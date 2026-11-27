    pub(super) fn native_client_chest_cleaner_context(&self)->Option<String> {
        if !self.client_auto_tool_player() || self.title.is_some() {return None;}
        let (menu,title)=self.client_container_menu_text(self.screen)?;
        let translated:Vec<Vec<String>>=super::chest_cleaner_titles::TITLE_KEYS.iter().map(|keys|
            keys.iter().map(|key|horizon_client::text::Text::translate(key,&[]).plain()).collect()).collect();
        Some(serde_json::json!({"menu":menu,"title":title.plain(),"translated":translated}).to_string())
    }
