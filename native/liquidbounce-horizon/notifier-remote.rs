pub(super) fn client_notifier_stack(
    &self,
    stack: &horizon_client::items::ItemStack,
) -> serde_json::Value {
    use serde_json::json;
    let components = self
        .pack
        .item_renderer
        .models
        .components_view(&horizon_client::item_render::stack_view(stack))
        .to_map();
    let tracked = horizon_client::item_sync::consumable(stack, self.items()).is_some_and(|c| {
        matches!(
            c.animation,
            horizon_world::consumable::UseAnimation::Eat
                | horizon_world::consumable::UseAnimation::Drink
        )
    });
    json!({"id":stack.item.as_ref(),"count":stack.count,"components":json!([stack.item.as_ref(),components]).to_string(),"name":self.stack_hover_name(stack),"duration":self.use_duration(stack),"consumable":tracked})
}
pub(super) fn client_notifier_snapshot(&self) -> serde_json::Value {
    use serde_json::json;
    let Some(r) = self.remote.as_ref() else {
        return json!({"online":[],"players":[]});
    };
    let online = r
        .tab_players
        .iter()
        .map(|(uuid, p)| json!({"uuid":format!("{uuid:032x}"),"name":p.name,"mode":p.game_mode}))
        .collect::<Vec<_>>();
    let players=r.report_helper_player_order.iter().filter_map(|id|{
            if *id==r.entity_id{return None;}
            let m=self.mobs.iter().find(|m|m.id as i32==*id&&!m.removed&&m.kind.id=="minecraft:player")?;
            let uuid=r.level_players.get(id)?;let name=self.remote_player_name(*id as u32)?;
            let hands=[0,1].map(|slot|horizon_client::mobs::finalize::item_stack(m,slot).filter(|s|s.count>0).map(|s|self.client_notifier_stack(&s)));
            Some(json!({"id":id,"uuid":format!("{uuid:032x}"),"name":name,"using":m.extra.using_item>0,"activeHand":usize::from(m.extra.using_off_hand),"ticksUsing":m.extra.using_item.saturating_sub(1),"hands":hands}))
        }).collect::<Vec<_>>();
    json!({"online":online,"players":players})
}
pub(super) fn client_notifier_packet(&mut self, packet: &C) {
    use serde_json::json;
    if !self.native_window() {
        return;
    }
    // Before the ordinary handler: actor execution order matches mc.execute.
    let value = match packet {
        C::PlayerInfoUpdate(p) => {
            json!({"kind":"info","actions":p.actions,"entries":p.entries.iter().map(|p|json!({"uuid":format!("{:032x}",p.uuid),"name":p.name,"mode":p.game_mode})).collect::<Vec<_>>() })
        }
        C::PlayerInfoRemove(p) => {
            json!({"kind":"remove","ids":p.uuids.iter().map(|id|format!("{id:032x}")).collect::<Vec<_>>() })
        }
        C::EntityEvent(p) if matches!(p.event, 3 | 35) => {
            let Some(uuid) = self.client_murder_mystery_uuid(p.id) else {
                return;
            };
            let local = self.remote.as_ref().is_some_and(|r| r.entity_id == p.id);
            let name = if local {
                self.own_profile_name()
            } else {
                self.remote_player_name(p.id as u32).unwrap_or_default().to_owned()
            };
            json!({"kind":"entity","uuid":uuid,"name":name,"local":local,"event":p.event})
        }
        C::RemoveEntities(p) => {
            json!({"kind":"removeEntities","ids":p.ids.iter().filter_map(|id|self.client_murder_mystery_uuid(*id)).collect::<Vec<_>>() })
        }
        _ => return,
    };
    self.native_client_notifier(&value.to_string());
}
