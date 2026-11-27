use crate::gui_closer::{Filter,GuiCloser,Mode};
use serde_json::Value;
#[test]
fn unchanged_kotlin_module_and_java_patterns_match_native_decisions() {
 let inputs:Value=serde_json::from_str(include_str!("inputs.json")).unwrap();
 let source:Value=serde_json::from_str(include_str!("source-oracle.json")).unwrap();
 let mut differences=Vec::new();
 for (i,(v,expected)) in inputs.as_array().unwrap().iter().zip(source.as_array().unwrap()).enumerate() {
  let filters:Result<Vec<_>,_>=v["patterns"].as_array().unwrap().iter().map(|p|p.as_str().map(Filter::new).transpose()).collect();
  if filters.is_ok()!=expected["valid"].as_bool().unwrap(){differences.push(format!("compile vector {i}: {v}, native valid={}",filters.is_ok()));continue;}
  let Ok(filters)=filters else {continue};
  let state=GuiCloser {mode:Mode::parse(v["mode"].as_str().unwrap()).unwrap(),filters,print_screen_title:v["print"].as_bool().unwrap()};
  let decision=state.screen_opening(v["running"].as_bool().unwrap(),v["screen"]=="container",v["title"].as_str().unwrap());
  if decision.cancel!=expected["cancel"].as_bool().unwrap()||decision.print_title!=expected["print"].as_bool().unwrap()||decision.null_filter!=expected["nullFilter"].as_bool().unwrap(){differences.push(format!("decision vector {i}: {v}, native={decision:?}, source={expected}"));continue;}
  if decision.print_title {
   let messages=expected["messages"].as_array().unwrap();assert_eq!(messages.len(),1);
   assert_eq!(messages[0][0]["text"],"GUICloser: container screen title: ");assert_eq!(messages[0][0]["kind"],"regular");
   assert_eq!(messages[0][1]["text"],v["title"]);assert_eq!(messages[0][1]["copied"],v["title"]);assert_eq!(messages[0][1]["kind"],"highlight");
  }
 }
 assert!(differences.is_empty(),"{}",differences.join("\n"));
}
#[test]
fn defaults_and_choice_lookup_retain_originals() {
 let state=GuiCloser::default();assert_eq!(state.mode,Mode::Matches);
 assert_eq!(state.filters[0].as_ref().unwrap().pattern(),"^Vote$");assert!(!state.print_screen_title);
 for value in ["Matches","MATCHES","matcheſ"] {assert_eq!(Mode::parse(value),Some(Mode::Matches));}
 for value in ["Contains","CONTAINS","Contaıns","CONTAİNS"] {assert_eq!(Mode::parse(value),Some(Mode::Contains));}
 for value in ["bad"," Contains","Contains "] {assert_eq!(Mode::parse(value),None);}
}
#[test]
fn actual_jdk_supplementary_search_selection_matches_native() {
 let rows:Value=serde_json::from_str(include_str!("jdk-search-layout.json")).unwrap();
 let mut differences=Vec::new();
 for row in rows.as_array().unwrap() {
  let pattern=row["pattern"].as_str().unwrap();
  let native=java_regex::Regex::new(pattern).unwrap();
  if native.uses_supplementary_search()!=row["hasSupplementary"].as_bool().unwrap(){differences.push(format!("{row}, native supplementary={}",native.uses_supplementary_search()));}
  let mut actual=native.lookbehind_uses_supplementary();actual.sort();
  let mut expected:Vec<_>=row["lookbehinds"].as_array().unwrap().iter().map(|node|node["kind"].as_str().unwrap().ends_with('S')).collect();expected.sort();
  if actual!=expected {differences.push(format!("{row}, native lookbehind supplementary={actual:?}"));}
 }
 assert!(differences.is_empty(),"{}",differences.join("\n"));
}
