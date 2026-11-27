    pub(super) fn native_client_anti_cheat_detect_chat(&mut self,tag:&str,certainty:i32){
        use horizon_client::text::Text;
        if !self.native_window(){return;}
        let mut text=Text::empty();
        for(index,c)in "LiquidBounce".chars().enumerate(){
            let f=index as f32/11.0;
            let ch=|a:f32,b:f32|f.mul_add(b-a,a)as u32;
            text=text.append(Text::colored(c.to_string(),(ch(70.,36.)<<16)|(ch(119.,170.)<<8)|ch(255.,127.)));
        }
        let language=horizon_client::lang::code().to_ascii_lowercase().replace('-',"_");
        let (before,middle,after)=match language.as_str(){
            "zh_cn"|"zh_sg"|"zh_hans"=>(" ▸ 检测到 "," 反作弊（确信度 ","）。"),
            "zh_tw"|"zh_hk"|"zh_mo"|"zh_hant"=>(" ▸ 偵測到 "," 反作弊（確信度 ","）。"),
            _=>(" ▸ Detected "," AntiCheat ("," certainty)."),
        };
        text=text.append(Text::colored(before,0xaaaaaa)).append(Text::colored(tag,0xffaa00))
            .append(Text::colored(middle,0xaaaaaa)).append(Text::colored(format!("{certainty}%"),0xffaa00)).append(Text::colored(after,0xaaaaaa));
        let focused=self.screen==Screen::Chat;let ticks=self.visual_ticks();
        self.ui.chat.add(&self.gui.atlas,text,None,ChatSource::Client,ticks,focused);
    }
