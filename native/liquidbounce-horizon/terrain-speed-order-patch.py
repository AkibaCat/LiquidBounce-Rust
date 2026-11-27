#!/usr/bin/env python3
"""Actual ClientLevel entity turns, selected native solvers and passenger order."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def method(text,signature):
    start=text.index(signature);cursor=text.index('{',start)+1;depth=1
    while depth:
        depth+=(text[cursor]=='{')-(text[cursor]=='}');cursor+=1
    return text[start:cursor]

def apply(edit,once):
    edit('crates/horizon-client/src/terrain_speed_order.rs',lambda _:(HERE/'terrain-speed-order.rs').read_text(encoding='utf-8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod terrain_speed;\n','pub mod terrain_speed;\npub mod terrain_speed_order;\n'))
    def remote(t):
        t=once(t,'    ticked_before_player: std::collections::HashSet<u32>,\n','    ticked_before_player: std::collections::HashSet<u32>,\n    terrain_order:horizon_client::terrain_speed_order::Order,\n')
        assert t.count('            ticked_before_player: std::collections::HashSet::new(),')==2
        t=t.replace('            ticked_before_player: std::collections::HashSet::new(),','            ticked_before_player: std::collections::HashSet::new(),\n            terrain_order:Default::default(),')
        t=once(t,'                r.entities.clear();','                r.entities.clear();\n                r.terrain_order=Default::default();')
        t=once(t,'            r.entities.insert(\n                id,\n                Tracked {','            r.terrain_order.added(id as u32);\n            r.entities.insert(\n                id,\n                Tracked {')
        t=once(t,'                    r.entity_id = p.player_id;','                    r.entity_id = p.player_id;\n                    r.terrain_order.player_added();')
        t=once(t,'                    r.ticked_before_player = before;','                    r.ticked_before_player = before;\n                    r.terrain_order.player_added();')
        t=once(t,'            seat_passengers(&mut r.entities, vehicle, passengers);','            seat_passengers(&mut r.entities, vehicle, passengers);\n            r.terrain_order.set_passengers(vehicle,passengers);')
        t=once(t,'        self.tick_own_rockets();\n','        if !horizon_client::terrain_speed_bridge::ordered(){self.tick_own_rockets();}\n')
        # Keep the exact already-composed local player body, including other
        # modules' event seams. The frame executes it from the entity walk.
        original=method(t,'    fn tick_remote_level(&mut self) -> Result<()> {')
        timer='let client_player_cancelled = self.client_timer_player_tick_ready();' in original
        legacy=original
        if timer:
            original=once(original,'let client_player_cancelled = self.client_timer_player_tick_ready();','let client_player_cancelled=if horizon_client::terrain_speed_bridge::ordered(){false}else{self.client_timer_player_tick_ready()};')
            original=once(original,'if !client_player_cancelled { self.cooldowns.tick(); }','if !horizon_client::terrain_speed_bridge::ordered()&&!client_player_cancelled{self.cooldowns.tick();}')
            original=once(original,'if !hold && !client_player_cancelled {','if !hold&&!horizon_client::terrain_speed_bridge::ordered()&&!client_player_cancelled{')
            original=once(original,'if !client_player_cancelled { self.tick_client_effects(); }','if !horizon_client::terrain_speed_bridge::ordered()&&!client_player_cancelled{self.tick_client_effects();}')
            original=once(original,'if !client_player_cancelled { self.sleep_timer = super::tick_sleep_timer(self.sleep_timer, self.sleeping.is_some()); }','if !horizon_client::terrain_speed_bridge::ordered()&&!client_player_cancelled{self.sleep_timer=super::tick_sleep_timer(self.sleep_timer,self.sleeping.is_some());}')
        local_start=original.index('        let entity_collisions = self.player_entity_collisions();')
        local=original[local_start:original.rfind('        Ok(())')]
        local=once(local,'        let pushed = pushed_after.add_vector(self.remote_pushes(true));','        let pushed=Vec3::ZERO;')
        local=once(local,'            player.velocity = player.velocity.add_vector(pushed);','            if !horizon_client::terrain_speed_bridge::ordered(){player.velocity=player.velocity.add_vector(pushed);}')
        # In legacy mode retain its old accumulated-push path, while ordered
        # mode applies each real entity impulse at that entity's turn.
        local=once(local,'        let pushed=Vec3::ZERO;','        let pushed=if horizon_client::terrain_speed_bridge::ordered(){Vec3::ZERO}else{pushed_after.add_vector(self.remote_pushes(true))};')
        chunk=method(local,'        if !self.streaming.is_lit(cx, cz) {')
        local=once(local,chunk,'''        if !self.streaming.is_lit(cx,cz){
            return Ok(self.remote.as_ref().filter(|r|r.loaded_sent).map(|r|r.last_input));
        }
''')
        if timer:
            timer_body='''        let client_player_cancelled=if horizon_client::terrain_speed_bridge::ordered(){
            let cancelled=self.client_timer_player_tick_ready();
            let hold=!self.clock.world_runs()&&self.clock.still();
            if !cancelled{
                self.cooldowns.tick();
                if !hold{for pop in &mut self.hotbar_pop{*pop=pop.saturating_sub(1);}}
                if !hold{self.tick_client_effects();}
                self.sleep_timer=super::tick_sleep_timer(self.sleep_timer,self.sleeping.is_some());
            }
            cancelled
        }else{client_player_cancelled};
'''
            local=once(local,'        if client_player_cancelled {',timer_body+'        if client_player_cancelled {')
            cancel=method(local,'        if client_player_cancelled {')
            local=once(local,cancel,'''        if client_player_cancelled{
            return Ok(self.remote.as_ref().filter(|r|r.loaded_sent).map(|r|r.last_input));
        }''')
            local=once(local,'        self.native_terrain_player_tick();\n','')
        local=local.replace('            return Ok(());','            return Ok(None);')
        local=once(local,'        self.remote_send_state(input | u8::from(auto_jumped) << 4);','        let terrain_input=input | u8::from(auto_jumped) << 4;')
        local+='        Ok(Some(terrain_input))\n'
        signature='    fn native_terrain_tick_player(&mut self,pushed_after:Vec3'+(',client_player_cancelled:bool' if timer else '')+')->Result<Option<u8>>{\n'
        player=signature+local+'    }\n'
        legacy_args='terrain_legacy_push'+(',client_player_cancelled' if timer else '')
        frame=original[:local_start]+'        if !terrain_ordered{terrain_input=self.native_terrain_tick_player('+legacy_args+')?;}\n'+'''
        if let Some(input)=terrain_input{self.remote_send_state(input);}
        Ok(())
    }'''
        frame=once(frame,'        let pushed_after = self.remote_pushes(false);','        let terrain_ordered=horizon_client::terrain_speed_bridge::ordered();\n        let terrain_legacy_push=if terrain_ordered{Vec3::ZERO}else{self.remote_pushes(false)};\n        let mut terrain_input=None;')
        frame=once(frame,'        self.tick_remote_entities(running);\n        self.native_terrain_flush();','''        if terrain_ordered{
            let mut ticker=TerrainEntityTicker{game:self,input:None};
            horizon_client::terrain_speed_order::tick_entities(&mut ticker,running)?;
            terrain_input=ticker.input;
            let _ancillary=horizon_client::terrain_speed_bridge::select(horizon_client::terrain_speed_order::Selection::Ancillary);
            self.tick_remote_entities(running);
        }else{self.tick_remote_entities(running);}
        self.native_terrain_flush();''')
        t=once(t,legacy,frame+'\n'+player+'\n'+(HERE/'terrain-speed-scheduler.rs').read_text(encoding='utf-8'))
        # Select the actual body in the existing solver. Preparatory snapshots
        # may read other bodies, but no unselected body advances twice.
        body=method(t,'    fn tick_remote_entities(&mut self, running: bool) {')
        body=once(body,'        if running {\n            self.tick_text_displays();','        if running&&horizon_client::terrain_speed_bridge::ancillary(){\n            self.tick_text_displays();')
        body=once(body,'            let m = &mut self.mobs[i];\n            let fluids = fluids[i].take();','            let m = &mut self.mobs[i];\n            if !horizon_client::terrain_speed_bridge::selected(m.id){continue;}\n            let fluids = fluids[i].take();')
        body=once(body,'            let m = &mut self.mobs[i];\n            let rain = rain[i];','            let m = &mut self.mobs[i];\n            if !horizon_client::terrain_speed_bridge::selected(m.id){continue;}\n            let rain = rain[i];')
        body=once(body,'            self.orbs.retain_mut(|o| {','            self.orbs.retain_mut(|o| {\n                if !horizon_client::terrain_speed_bridge::selected(o.id){return true;}')
        body=once(body,'            items.retain_mut(|item| {','            items.retain_mut(|item| {\n                if !horizon_client::terrain_speed_bridge::selected(item.id){return true;}')
        body=once(body,'        let _ = self.handle_mob_events_reusable(&mut mob_fx);','''        if let horizon_client::terrain_speed_order::Selection::Entity(id)=horizon_client::terrain_speed_bridge::selection()
            &&self.mobs.iter().any(|m|m.id==id){self.native_terrain_entity_push(id);}
        let _ = self.handle_mob_events_reusable(&mut mob_fx);''')
        body=once(body,'        self.flash = self.flash.saturating_sub(1);','        if horizon_client::terrain_speed_bridge::ancillary(){self.flash=self.flash.saturating_sub(1);}')
        body=once(body,'            self.guardian_bubbles();','            if horizon_client::terrain_speed_bridge::ancillary(){self.guardian_bubbles();}')
        body=once(body,'            self.tick_remote_dragon(dragon_lerp);','            if horizon_client::terrain_speed_bridge::ancillary(){self.tick_remote_dragon(dragon_lerp);}')
        # Reading the dragon's interpolation must not consume it for every body.
        body=once(body,'            let dragon_lerp = self\n','            let dragon_lerp = self\n')
        body=once(body,'                .filter(|l| l.steps > 0)\n                .map(|l| {','                .filter(|l| l.steps > 0&&horizon_client::terrain_speed_bridge::ancillary())\n                .map(|l| {')
        body=once(body,'                for a in &mut self.armor_stands {','                for a in &mut self.armor_stands {\n                    if !horizon_client::terrain_speed_bridge::selected(a.id){continue;}')
        body=once(body,'            self.tick_remote_projectiles(&mut step);','            if horizon_client::terrain_speed_bridge::ancillary(){self.tick_remote_projectiles(&mut step);}')
        body=once(body,'            self.tick_bolts();','            if horizon_client::terrain_speed_bridge::ancillary(){self.tick_bolts();}')
        t=once(t,method(t,'    fn tick_remote_entities(&mut self, running: bool) {'),body)
        ticker=(HERE/'terrain-speed-ticker.rs').read_text(encoding='utf-8')
        if timer:ticker=once(ticker,'native_terrain_tick_player(Vec3::ZERO)?','native_terrain_tick_player(Vec3::ZERO,false)?')
        return t+'\n'+ticker
    edit('crates/horizon-client/src/game/remote.rs',remote)
    def player_tick(t):
        if 'pub(super) fn client_timer_player_tick(&self)' not in t:return t
        t=once(t,'pub(super) fn client_timer_player_tick(&self)','pub(super) fn client_timer_player_tick(&mut self)')
        t=once(t,'pub(super) fn client_timer_player_tick_ready(&self)','pub(super) fn client_timer_player_tick_ready(&mut self)')
        return once(t,'        self.native_client_player_tick(&mut cancelled);\n        self.native_client_player_tick_final(cancelled);','        self.native_client_player_tick(&mut cancelled);\n        self.native_terrain_player_tick();\n        self.native_client_player_tick_final(cancelled);')
    edit('crates/horizon-client/src/game/native_hooks.rs',player_tick)
    def objects(t):
        body=method(t,'    pub(super) fn tick_remote_objects(')
        for ty,ident in [('carts','c'),('boats','b'),('armor_stands','a')]:
            anchor=f'        for {ident} in &mut self.{ty} {{'
            body=once(body,anchor,anchor+f'\n            if !horizon_client::terrain_speed_bridge::selected({ident}.id){{continue;}}')
        return once(t,method(t,'    pub(super) fn tick_remote_objects('),body)
    edit('crates/horizon-client/src/game/remote_objects.rs',objects)
    edit('crates/horizon-client/src/game/remote_removal.rs',lambda t:once(t,'                r.entities.remove(&id);','                r.entities.remove(&id);\n                r.terrain_order.removed(id as u32);'))
    def vehicles(t):
        # Actual vehicle simulation is separate from the following rider input
        # latch. The scheduler executes the former at the vehicle's turn.
        body=method(t,'    fn drive_mount(&mut self, i: usize, keys: Keys, jump: f32) {')
        cut=body.index('        // LocalPlayer.aiStep, after the vehicle\'s tick.')
        vehicle=body[body.index('\n')+1:cut]
        # Events belong to the vehicle tick, before its passenger's input latch.
        vehicle+='        let _ = self.handle_mob_events(events);\n'
        rest=body[cut:body.rfind('    }')]
        rest=once(rest,'        let (yaw, pitch) = (self.yaw, self.pitch);','        let (yaw, pitch) = (self.yaw, self.pitch);') if '        let (yaw, pitch) = (self.yaw, self.pitch);' in rest else '        let (yaw,pitch)=(self.yaw,self.pitch);\n        let steer=self.steers_mount(&self.mobs[i]);\n'+rest
        rest=once(rest,'        let _ = self.handle_mob_events(events);','')
        extended='''    fn native_terrain_tick_mount_vehicle(&mut self,i:usize){
'''+vehicle+'''    }
    fn drive_mount(&mut self,i:usize,keys:Keys,jump:f32){
        if !horizon_client::terrain_speed_bridge::was_driven(self.mobs[i].id){self.native_terrain_tick_mount_vehicle(i);}
'''+rest+'''    }'''
        t=once(t,body,extended)
        helper='''    pub(super) fn native_terrain_tick_controlled_vehicle(&mut self,id:u32){
        if self.riding!=Some(id){return;}
        if let Some(i)=self.mobs.iter().position(|m|m.id==id)&&self.drives(&self.mobs[i]){
            self.native_terrain_tick_mount_vehicle(i);horizon_client::terrain_speed_bridge::mark_driven(id);
        }else if let Some(i)=self.boats.iter().position(|b|b.id==id)&&self.controls_boat(id){
            let boat=self.boats[i].clone();self.boats[i]=self.tick_boat(boat,true);
            horizon_client::terrain_speed_bridge::mark_driven(id);
        }
    }
'''
        t=once(t,'    pub(super) fn remote_ride(',helper+'    pub(super) fn remote_ride(')
        t=once(t,'                b = self.tick_boat(b, true);','                if !horizon_client::terrain_speed_bridge::was_driven(id){b=self.tick_boat(b,true);}')
        body=method(t,'    pub(super) fn remote_ride(')
        assert body.count('p.velocity = Vec3::ZERO;')==2
        body=body.replace('p.velocity = Vec3::ZERO;','if !horizon_client::terrain_speed_bridge::ordered(){p.velocity=Vec3::ZERO;}')
        t=once(t,method(t,'    pub(super) fn remote_ride('),body)
        t=once(t,'        for i in 0..self.boats.len() {\n            let driven = self.controls_boat(self.boats[i].id);','        for i in 0..self.boats.len() {\n            if !horizon_client::terrain_speed_bridge::selected(self.boats[i].id){continue;}\n            let driven = self.controls_boat(self.boats[i].id);')
        return t
    edit('crates/horizon-client/src/game/remote_vehicles.rs',vehicles)
    def rockets(t):
        t=once(t,'        if !self.clock.runs_normally()\n            || self.player.is_none()','        if (!self.clock.runs_normally()&&!horizon_client::terrain_speed_bridge::ordered())\n            || self.player.is_none()')
        t=once(t,'        for r in rockets.iter_mut().filter(|r| r.attached_to == Some(me)) {','''        for r in rockets.iter_mut().filter(|r|r.attached_to==Some(me)&&horizon_client::terrain_speed_bridge::selected(r.id)){
            if horizon_client::terrain_speed_bridge::ordered(){r.remote_step(&mut SceneCollisions{world:&self.world,binding:self.collision,subject:None});}''')
        t=once(t,'        for r in &mut rockets {\n            r.remote_step(','''        for r in &mut rockets {
            if horizon_client::terrain_speed_bridge::ordered()&&r.attached_to==me{continue;}
            r.remote_step(''')
        return t
    edit('crates/horizon-client/src/game/remote_projectiles.rs',rockets)
