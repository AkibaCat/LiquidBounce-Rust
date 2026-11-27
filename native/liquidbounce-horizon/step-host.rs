// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_step_input(&mut self,can_one:bool,can_one_quarter:bool,can_one_half:bool,jump:&mut bool,effects:&mut String){
        mod_api::dispatch_client_step_input(self,mod_api::mods(),can_one,can_one_quarter,can_one_half,jump,effects);
    }
    pub(super) fn native_client_step_effects(&mut self,value:&str){self.step_effects(value);}
