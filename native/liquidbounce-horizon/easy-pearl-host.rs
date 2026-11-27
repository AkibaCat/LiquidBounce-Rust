pub(super) fn native_client_easy_pearl(&self,input:&str,output:&mut String){
    if self.native_window()&&self.native_client_slot_owner(){mod_api::dispatch_client_easy_pearl(self,mod_api::mods(),input,output);}
}
pub(super) fn native_client_easy_pearl_query(&self,text:&str)->String{self.client_easy_pearl_query(text)}
