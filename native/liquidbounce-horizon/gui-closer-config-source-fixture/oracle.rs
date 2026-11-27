use super::*;
use liquidbounce_behaviors::gui_closer::GuiCloser;

#[test]
fn unchanged_list_adapter_and_group_serializers_match_native_config() {
    let inputs:Vec<Value>=serde_json::from_str(include_str!("inputs.json")).unwrap();
    let expected:Vec<Value>=serde_json::from_str(include_str!("expected.json")).unwrap();
    let common=CommonSettings{enabled:false,hidden:false,key_bind:json!({})};
    for (input,expected) in inputs.iter().zip(&expected) {
        let mut state=GuiCloser::default();
        let patch=parse_settings(ModuleKind::GuiCloser,&json!({"name":"GUICloser","value":[{"name":"Filter","value":input}]})).unwrap();
        assert_eq!(!patch.issues.is_empty(),expected["failed"].as_bool().unwrap(),"{input}");
        for change in patch.changes { if let SettingChange::GuiCloser(v)=change {v.apply(&mut state);} }
        for (format,field) in [(JsonFormat::File,"file"),(JsonFormat::Interop,"interop")] {
            let actual=serialize_gui_closer(&state,&common,format,&str::to_owned).unwrap();
            let n=nodes(&actual,"GUICloser","GUICloser").unwrap();
            assert_eq!(n["Filter"]["value"],expected["filter"],"{input}");
            let e=nodes(&expected[field],"GUICloser","GUICloser").unwrap();
            assert_eq!(n["Filter"]["value"],e["Filter"]["value"]);
            assert!(e.contains_key("PrintScreenTitle"));
            assert_eq!(n["Filter"].get("innerValueType"),e["Filter"].get("innerValueType"));
            assert_eq!(n["Filter"].get("valueType"),e["Filter"].get("valueType"));
            assert!(e["Filter"].get("innerType").is_none());
        }
        let public=nodes(&expected["public"],"GUICloser","GUICloser").unwrap();
        assert!(!public.contains_key("PrintScreenTitle"));
    }
}
