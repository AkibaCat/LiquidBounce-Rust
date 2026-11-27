// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared single-thread game context and the original TickBody key deferral.
use super::input::Input;
use std::{cell::Cell,rc::Rc};
#[derive(Default)]
struct TickState { moving:Cell<bool>, stand:Cell<bool>, pressed:Cell<Option<Input>>,exact:Cell<bool> }
#[derive(Clone,Default)]
pub struct NavigationContext { state:Rc<TickState> }
impl NavigationContext {
    pub fn is_moving(&self)->bool{self.state.moving.get()}
    pub fn set_moving(&self,moving:bool){self.state.moving.set(moving);}
    pub fn should_stand(&self)->bool{self.state.stand.get()}
    pub fn set_should_stand(&self,stand:bool){self.state.stand.set(stand);}
    pub fn can_click_inventory(&self,sprinting:bool)->bool{!self.is_moving()&&!sprinting}
    /// Every controller uses this; only Navigator commits one body input.
    pub fn press(&self,input:Input,exact:bool){self.state.pressed.set(Some(input));self.state.exact.set(exact);}
    pub fn take_pressed(&self)->Option<Input>{self.state.pressed.take()}
    pub fn pressed_exact(&self)->bool{self.state.exact.get()}
}
