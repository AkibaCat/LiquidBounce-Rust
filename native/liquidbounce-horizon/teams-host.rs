    pub(super) fn client_teams_facts(&self, entity:i32)->serde_json::Value {
        use horizon_client::{text::Text,scoreboard as sb,mobs::finalize};
        use horizon_world::player_inventory::EquipmentSlot as S;
        use serde_json::{Value,json};
        let Some(_player)=self.player.as_ref().filter(|p|p.native_client_queries_allowed()) else{return Value::Null;};
        if !self.native_window()||self.lan_acting.is_some()||self.lan_acting_uuid.is_some(){return Value::Null;}
        let color=|stack:Option<&ItemStack>|->Option<u32>{
            let view=horizon_client::item_render::stack_view(stack?);
            let c=self.pack.item_renderer.models.components_view(&view);
            c.get("minecraft:dyed_color").and_then(|v|v.as_i64().or_else(||v.get("rgb").and_then(Value::as_i64))).map(|n|n as u32|0xff000000)
        };
        let facts=|team:Option<&sb::Team>,name:Text,armor:[Option<u32>;4],player:bool|{
            let text=team.map_or_else(||name.clone(),|t|t.formatted_name(name.clone()));
            let root=text.component();
            let root_color=root.get("color").and_then(|v|v.as_str()).and_then(|s|s.strip_prefix('#').and_then(|v|u32::from_str_radix(v,16).ok()).or_else(||horizon_client::text::named_color(s)));
            let name_color=root_color.or_else(||text.resolve_legacy().spans.iter().filter(|s|!s.text.is_empty()).find_map(|s|s.style.color));
            json!({"team":team.map(|t|&t.name),"teamColor":team.and_then(|t|t.color.as_deref()).and_then(sb::team_color_rgb),"name":text.plain(),"nameColor":name_color,"armor":armor,"player":player})
        };
        let own_name=self.own_profile_name();let own_team=self.score.board.players_team(&own_name);
        let own=facts(own_team,Text::literal(own_name),std::array::from_fn(|i|color(self.armor[i].as_ref())),true);
        let local_id=self.remote.as_ref().map_or(0,|r|r.entity_id);
        let other=if entity==local_id {own.clone()}
        else if let Some(g)=self.lan.iter().flat_map(|h|h.hub.guests.iter()).find(|g|g.placed&&g.entity_id==entity&&g.dimension==self.dim.id()){
            facts(self.score.board.players_team(&g.name),Text::literal(g.name.clone()),std::array::from_fn(|i|color(g.armor[i].as_ref())),true)
        } else if let Some(m)=self.mobs.iter().find(|m|m.id as i32==entity){
            let player=horizon_client::command::args::normalize(&m.kind.id)=="minecraft:player";
            let name=self.remote_player_name(m.id);
            let team=if player{name.as_deref().and_then(|n|self.score.board.players_team(n))}else{self.mob_team_of(m)};
            let slots=[S::Feet,S::Legs,S::Chest,S::Head];
            let armor=std::array::from_fn(|i|color(finalize::equipment_stack(m,slots[i]).as_ref()));
            facts(team,name.map_or_else(||m.custom_name_text.clone().unwrap_or_else(||m.command_name()),Text::literal),armor,player)
        } else if let Some(a)=self.armor_stands.iter().find(|a|a.id as i32==entity){
            let name=a.custom_name.clone().unwrap_or_else(||Text::tr("entity.minecraft.armor_stand",&[]));
            facts(self.mob_team(entity as u32),name,std::array::from_fn(|i|color(a.equipment[i+2].as_ref())),false)
        } else {
            // Non-living entities still receive their actual scoreboard color.
            facts(self.mob_team(entity as u32),Text::empty(),[None;4],false)
        };
        json!({"own":own,"other":other})
    }
