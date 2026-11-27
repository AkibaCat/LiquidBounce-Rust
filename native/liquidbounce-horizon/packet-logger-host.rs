    pub(super) fn native_client_packet_logger_chat(&mut self,payload:&str){
        use horizon_client::text::{Style,ClickEvent};
        let Ok(serde_json::Value::Array(parts))=serde_json::from_str(payload)else{return;};
        let mut text=Text::empty();
        for part in parts{
            let mut style=Style::color(part["color"].as_u64().unwrap_or(0xffffff)as u32);
            style.bold=part["bold"].as_bool();
            if let Some(s)=part["copy"].as_str(){style.click=Some(ClickEvent::CopyToClipboard(s.into()));}
            text=text.append(Text::styled(part["text"].as_str().unwrap_or(""),style));
        }
        let ticks=self.visual_ticks();let focused=self.screen==Screen::Chat;
        self.ui.chat.add(&self.gui.atlas,text,None,ChatSource::Client,ticks,focused);
    }
