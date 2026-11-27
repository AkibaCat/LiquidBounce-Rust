// SPDX-License-Identifier: GPL-3.0-or-later
//! ClientLevel entity insertion order and ordered passenger lists.
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    #[default]
    All,
    Entity(u32),
    Ancillary,
}
impl Selection {
    pub fn includes(self, id: u32) -> bool {
        matches!(self,Self::All) || matches!(self,Self::Entity(wanted) if wanted==id)
    }
    pub fn ancillary(self) -> bool {
        matches!(self,Self::All|Self::Ancillary)
    }
}
#[derive(Clone, Debug, Default)]
pub struct Order {
    insertion: Vec<Option<u32>>,
    passengers: HashMap<u32, Vec<u32>>,
}
impl Order {
    pub fn player_added(&mut self) {
        self.insertion.retain(Option::is_some);
        self.insertion.push(None);
    }
    pub fn added(&mut self, id: u32) {
        self.insertion.retain(|entry| *entry != Some(id));
        self.insertion.push(Some(id));
    }
    pub fn removed(&mut self, id: u32) {
        self.insertion.retain(|entry| *entry != Some(id));
        self.passengers.remove(&id);
        for children in self.passengers.values_mut() {
            children.retain(|&child| child != id);
        }
    }
    pub fn set_passengers(&mut self, vehicle: u32, passengers: &[i32]) {
        let wanted: HashSet<u32> = passengers.iter().map(|&id| id as u32).collect();
        for children in self.passengers.values_mut() {
            children.retain(|child| !wanted.contains(child));
        }
        self.passengers.insert(vehicle,passengers.iter().map(|&id|id as u32).collect());
    }
    pub fn roots(&self, local:u32)->Vec<u32>{self.insertion.iter().map(|entry|entry.unwrap_or(local)).collect()}
    pub fn passengers(&self,id:u32)->Vec<u32>{self.passengers.get(&id).cloned().unwrap_or_default()}
}

/// Storage seam around the actual ClientLevel walk. Membership and vehicle
/// relationships are queried at each turn, including after a prior tick mutates
/// them; entities added during traversal enter the root list next frame.
pub trait EntityTicks {
    type Error;
    fn roots(&self)->Vec<u32>;
    fn removed(&self,id:u32)->bool;
    fn vehicle(&self,id:u32)->Option<u32>;
    fn ticking(&self,id:u32)->bool;
    fn player(&self,id:u32)->bool;
    fn passengers(&self,id:u32)->Vec<u32>;
    fn detach(&mut self,id:u32);
    /// Includes commonTick, then tick or rideTick (ZERO, tick, positionRider).
    fn tick(&mut self,id:u32,passenger:bool)->Result<(),Self::Error>;
}
pub fn tick_entities<T:EntityTicks>(entities:&mut T,running:bool)->Result<(),T::Error>{
    for id in entities.roots(){
        if entities.removed(id)||entities.vehicle(id).is_some()
            ||(!running&&!entities.player(id)&&!has_player_passenger(entities,id,&mut HashSet::new())){continue;}
        entities.tick(id,false)?;
        for child in entities.passengers(id){tick_passenger(entities,id,child)?;}
    }
    Ok(())
}
fn has_player_passenger<T:EntityTicks>(entities:&T,id:u32,seen:&mut HashSet<u32>)->bool{
    if !seen.insert(id){return false;}
    entities.passengers(id).into_iter().any(|child|entities.player(child)||has_player_passenger(entities,child,seen))
}
fn tick_passenger<T:EntityTicks>(entities:&mut T,vehicle:u32,id:u32)->Result<(),T::Error>{
    if entities.removed(id)||entities.vehicle(id)!=Some(vehicle){entities.detach(id);return Ok(());}
    if !entities.player(id)&&!entities.ticking(id){return Ok(());}
    entities.tick(id,true)?;
    for child in entities.passengers(id){tick_passenger(entities,id,child)?;}
    Ok(())
}
