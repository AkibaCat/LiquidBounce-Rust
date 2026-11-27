    /// Use the same command sender, including old-version acknowledgements.
    pub(super) fn native_client_plugins_command(&mut self,command:&str){if self.native_client_slot_owner(){self.send_command(command);}}
    pub(super) fn native_client_plugins_suggest(&mut self,id:i32,input:&str){
        if !self.native_client_slot_owner(){return;}
        let mut writer=horizon_protocol::buf::Writer::new();writer.varint(id);writer.string(input);
        self.send(horizon_protocol::packets::play::Serverbound::Unknown(horizon_protocol::ids::play::serverbound::COMMAND_SUGGESTION,writer.data));
    }
    pub(super) fn native_client_plugins_decode(&self,data:&[u8])->String{
        let mut reader=horizon_protocol::buf::Reader::new(data);
        let decoded=(||->horizon_protocol::Result<(i32,Vec<String>)>{
            let id=reader.varint()?;let _start=reader.varint()?;let _length=reader.varint()?;let count=reader.count(reader.remaining())?;
            let mut texts=Vec::with_capacity(count.min(4096));
            for _ in 0..count{let text=reader.string(32767)?;if reader.bool()?{let _tooltip=reader.nbt()?;}texts.push(text);}
            Ok((id,texts))
        })();
        decoded.map(|(id,texts)|serde_json::json!({"id":id,"texts":texts}).to_string()).unwrap_or_default()
    }
    pub(super) fn native_client_plugins_chat(&mut self,prefix:&str,names:Vec<String>,colors:Vec<u32>,error:bool){
        if !self.native_client_slot_owner(){return;}
        let mut line=Text::literal(prefix);
        if error {line=line.with_color(0xff5555);}
        for(index,name)in names.into_iter().enumerate(){if index!=0{line.push(", ");}line.push(Text::colored(name,colors.get(index).copied().unwrap_or(0xff5555)));}
        let focused=self.screen==Screen::Chat;let ticks=self.visual_ticks();let atlas=&self.gui.atlas;
        self.ui.chat.add(atlas,line,None,ChatSource::Client,ticks,focused);
    }
