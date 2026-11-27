/// A client renderer service. Implementations must never acquire the actor or call Game/Host.
/// Its matcher/settings live independently so text drawn while an actor callback is borrowed is safe.
pub trait NameProtection:Send+Sync {
    fn replace(&self,text:&str)->Option<String>;
    /// `text` is an already resolved FormattedCharSequence. No parsing or recursive substitution.
    fn wrap(&self,text:&Text)->Option<Text>;
}
fn name_protection_service()->&'static std::sync::RwLock<Option<std::sync::Arc<dyn NameProtection>>>{
    static SERVICE:std::sync::OnceLock<std::sync::RwLock<Option<std::sync::Arc<dyn NameProtection>>>>=std::sync::OnceLock::new();
    SERVICE.get_or_init(||std::sync::RwLock::new(None))
}
pub fn install_name_protection(service:std::sync::Arc<dyn NameProtection>){
    *name_protection_service().write().expect("name protection service")=Some(service);
}
pub fn protect_string(text:&str)->std::borrow::Cow<'_,str>{
    let service=name_protection_service().read().expect("name protection service").clone();
    service.and_then(|s|s.replace(text)).map_or(std::borrow::Cow::Borrowed(text),std::borrow::Cow::Owned)
}
pub fn protect_text(text:&Text)->std::borrow::Cow<'_,Text>{
    let service=name_protection_service().read().expect("name protection service").clone();
    service.and_then(|s|s.wrap(text)).map_or(std::borrow::Cow::Borrowed(text),std::borrow::Cow::Owned)
}
