// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{runtime::LiquidBounceHooks,targeting};
use horizon_client::mod_api::{Host,ClientTrueSightEntityEvent,ClientTrueSightMarkersEvent};
impl LiquidBounceHooks {
 pub(crate) fn true_sight_entity(&self,host:&dyn Host,event:&mut ClientTrueSightEntityEvent) {
  let Some(snapshot)=targeting::snapshot(&host.client_targeting_entity(event.entity)) else{return;};
  let colors={let state=self.state();let modules=state.behaviors.borrow();modules.true_sight.colors(modules.query_running(&state.registry,"TrueSight"),snapshot.entity.kind.living,snapshot.entity.invisible,modules.query_running(&state.registry,"ESP") && modules.esp.mode==3 && modules.esp.true_sight.contains(&event.entity))};
  if let Some([body,feature])=colors{event.body=body;event.feature=feature;event.apply=true;}
 }
 pub(crate) fn true_sight_markers(&self,_host:&dyn Host,event:&mut ClientTrueSightMarkersEvent){let state=self.state();let modules=state.behaviors.borrow();event.mask=modules.true_sight.markers(modules.query_running(&state.registry,"TrueSight"));}
}
