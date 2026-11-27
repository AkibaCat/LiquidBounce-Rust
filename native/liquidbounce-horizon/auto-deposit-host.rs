// SPDX-License-Identifier: GPL-3.0-or-later
    pub(super) fn native_client_auto_deposit_context(&mut self)->Option<String>{self.auto_deposit_context()}
    pub(super) fn native_client_auto_deposit_target(&self,value:&str)->Option<String>{self.auto_deposit_target(value)}
    pub(super) fn native_client_auto_deposit_punch(&mut self,value:&str)->bool{self.auto_deposit_punch(value)}
