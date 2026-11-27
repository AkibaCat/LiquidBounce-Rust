// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleDebug / DebugSection / DebugOverviewLayout source behavior.
use java_regex::{PatternSyntaxError, Regex};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct Filter(Regex);
impl Filter {
    pub fn new(v: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(v).map(Self)
    }
    pub fn pattern(&self) -> &str {
        self.0.pattern()
    }
}
impl PartialEq for Filter {
    fn eq(&self, o: &Self) -> bool {
        self.pattern() == o.pattern()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub parameters: bool,
    pub global: bool,
    pub geometry: bool,
    pub expires: i32,
    pub width: i32,
    pub title_filter: Filter,
    pub debug_log: bool,
    pub simulated_player: bool,
    pub ticks_to_predict: i32,
    pub graph: bool,
    pub curve: Vec<[f32; 2]>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            parameters: true,
            global: false,
            geometry: true,
            expires: 5,
            width: 180,
            title_filter: Filter::new(".*").unwrap(),
            debug_log: false,
            simulated_player: false,
            ticks_to_predict: 20,
            graph: false,
            curve: vec![[0., 120.], [50., 60.], [140., 120.], [180., 90.]],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High,
    Normal,
    Low,
}
impl Priority {
    pub fn parse(v: &str) -> Self {
        match v {
            "HIGH" => Self::High,
            "LOW" => Self::Low,
            _ => Self::Normal,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub value: Value,
    pub time: i64,
    pub stamp: u32,
    pub order: u64,
    pub priority: Priority,
}
#[derive(Clone, Debug)]
pub struct Section {
    pub root: String,
    pub label: Option<String>,
    pub path: String,
    pub accepted: bool,
    pub global: bool,
    pub parameters: BTreeMap<String, Parameter>,
    pub geometry: BTreeMap<String, Value>,
}
#[derive(Clone, Debug)]
pub enum Row {
    Header(String),
    Subheader(String),
    Item(Parameter, bool),
}
#[derive(Clone, Debug)]
pub struct Card {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rows: Vec<Row>,
}
#[derive(Default)]
pub struct Debug {
    pub settings: Settings,
    pub sections: BTreeMap<String, Section>,
    pub next_order: u64,
    pub structure_version: u64,
    pub scroll: f32,
    pub content_bottom: f32,
    pub content_right: f32,
    pub view_bottom: f32,
    pub predicted: Vec<[f64; 3]>,
    pub logging: bool,
    pub log: Vec<String>,
}
impl Debug {
    pub fn enabled(&mut self) {
        self.logging = self.settings.debug_log;
    }
    pub fn clear(&mut self) {
        self.sections.clear();
        self.structure_version = self.structure_version.wrapping_add(1);
        self.scroll = 0.;
        self.content_bottom = 22.;
        self.content_right = 120.;
        self.view_bottom = 22.;
        self.predicted.clear();
        self.logging = false;
        self.log.clear();
    }
    pub fn parameters_changed(&mut self) {
        for s in self.sections.values_mut() {
            s.parameters.clear();
        }
        self.structure_version = self.structure_version.wrapping_add(1);
    }
    pub fn global_changed(&mut self) {
        for s in self.sections.values_mut().filter(|s| s.global) {
            s.parameters.clear();
        }
        self.structure_version = self.structure_version.wrapping_add(1);
    }
    pub fn geometry_changed(&mut self) {
        for s in self.sections.values_mut() {
            s.geometry.clear();
        }
    }
    pub fn filter_changed(&mut self) {
        self.sections.clear();
        self.structure_version = self.structure_version.wrapping_add(1);
    }
    fn section(&mut self, path: &str, global: bool) -> &mut Section {
        let mut chain = path.split('/').collect::<Vec<_>>();
        if chain.len() > 16 {
            chain = chain[chain.len() - 16..].to_vec();
        }
        let path = chain.join("/");
        self.sections
            .entry(path.clone())
            .or_insert_with(|| Section {
                root: chain.first().unwrap_or(&"").to_string(),
                label: (chain.len() > 1).then(|| chain[1..].join(" / ")),
                accepted: self
                    .settings
                    .title_filter
                    .0
                    .contains_match(&path),
                path,
                global,
                parameters: BTreeMap::new(),
                geometry: BTreeMap::new(),
            })
    }
    pub fn put(
        &mut self,
        path: &str,
        global: bool,
        name: &str,
        priority: Priority,
        value: Value,
        now: i64,
    ) {
        if !self.settings.parameters || global && !self.settings.global {
            return;
        }
        if !self.section(path, global).accepted {
            return;
        }
        let created = !self.section(path, global).parameters.contains_key(name);
        let order = self.next_order;
        if created {
            self.next_order = self.next_order.wrapping_add(1);
            self.structure_version = self.structure_version.wrapping_add(1);
        }
        let s = self.section(path, global);
        let p = s.parameters.entry(name.into()).or_insert(Parameter {
            name: name.into(),
            value: Value::Null,
            time: 0,
            stamp: 0,
            order,
            priority,
        });
        let changed = p.priority != priority;
        p.priority = priority;
        p.value = value.clone();
        p.time = now;
        p.stamp = p.stamp.wrapping_add(1);
        if changed {
            self.structure_version = self.structure_version.wrapping_add(1);
        }
        if self.logging {
            self.log
                .push(format!("{now}\t{path}\t{name}\t{}\n", format_value(&value)));
        }
    }
    pub fn put_geometry(&mut self, path: &str, global: bool, name: &str, geometry: Value) {
        if !self.settings.geometry {
            return;
        };
        let s = self.section(path, global);
        if !s.accepted {
            return;
        }
        if geometry.is_null() {
            s.geometry.remove(name);
        } else {
            s.geometry.insert(name.into(), geometry);
        }
    }
    pub fn expire(&mut self, now: i64) {
        let earliest = now.saturating_sub(i64::from(self.settings.expires) * 1000);
        let mut removed = false;
        for s in self.sections.values_mut() {
            let n = s.parameters.len();
            s.parameters.retain(|_, p| p.time > earliest);
            removed |= n != s.parameters.len();
        }
        if removed {
            self.structure_version = self.structure_version.wrapping_add(1);
        }
    }
    pub fn scroll(&mut self, amount: f32, cursor: [f32; 2], chat: bool) -> bool {
        if !chat
            || !self.settings.parameters
            || self.content_bottom <= self.view_bottom
            || cursor[0] < 120.
            || cursor[0] > self.content_right
            || cursor[1] < 22.
            || cursor[1] > self.view_bottom
        {
            return false;
        }
        self.scroll -= amount * 24.;
        true
    }
    pub fn cards(&mut self, width: f32, height: f32) -> Vec<Card> {
        let left = 120.;
        let top: f32 = 22.;
        let right = width - 4.;
        let bottom = height - 50.;
        let w = (self.settings.width as f32).min(right - left);
        if w <= 0. {
            return Vec::new();
        }
        let max_columns = (((right - left + 2.) / (w + 2.)).floor() as usize).max(1);
        let mut groups: BTreeMap<String, Vec<&Section>> = BTreeMap::new();
        for s in self
            .sections
            .values()
            .filter(|s| s.accepted && !s.parameters.is_empty())
        {
            groups.entry(s.root.clone()).or_default().push(s);
        }
        let section_order = |s: &&Section| {
            (
                s.label.is_some(),
                s.parameters.values().map(|p| p.priority).min().unwrap(),
                s.parameters.values().map(|p| p.order).min().unwrap(),
            )
        };
        let mut groups = groups.into_iter().collect::<Vec<_>>();
        for (_, sections) in &mut groups {
            sections.sort_by_key(section_order);
        }
        groups.sort_by_key(|(_, ss)| {
            (
                ss.iter()
                    .flat_map(|s| s.parameters.values().map(|p| p.priority))
                    .min()
                    .unwrap(),
                ss.iter()
                    .flat_map(|s| s.parameters.values().map(|p| p.order))
                    .min()
                    .unwrap(),
            )
        });
        let rows = groups
            .into_iter()
            .map(|(title, ss)| {
                let mut rows = vec![Row::Header(title)];
                for s in ss {
                    if let Some(l) = &s.label {
                        rows.push(Row::Subheader(l.clone()));
                    }
                    let mut items = s.parameters.values().cloned().collect::<Vec<_>>();
                    items.sort_by_key(|p| (p.priority, p.order));
                    rows.extend(items.into_iter().map(|p| Row::Item(p, s.label.is_some())));
                }
                rows
            })
            .collect::<Vec<_>>();
        let place = |columns: usize| {
            let mut ends = vec![top; columns];
            let mut result = Vec::new();
            for rows in &rows {
                let column = (0..columns)
                    .min_by(|a, b| ends[*a].total_cmp(&ends[*b]).then(a.cmp(b)))
                    .unwrap();
                let h = rows.len() as f32 * 8. + 4.;
                result.push(Card {
                    x: left + column as f32 * (w + 2.),
                    y: ends[column],
                    width: w,
                    height: h,
                    rows: rows.clone(),
                });
                ends[column] += h + 2.;
            }
            let end = ends.into_iter().fold(top, f32::max) - if rows.is_empty() { 0. } else { 2. };
            (result, end.max(top))
        };
        let mut columns = 1;
        let (mut cards, mut end) = place(columns);
        while end > bottom && columns < max_columns {
            columns += 1;
            (cards, end) = place(columns);
        }
        self.content_bottom = end;
        self.content_right = cards.iter().map(|c| c.x + c.width).fold(left, f32::max);
        self.view_bottom = bottom;
        let cards_bottom = bottom - if end > bottom { 8. } else { 0. };
        self.scroll = self.scroll.clamp(0., (end - cards_bottom).max(0.));
        for c in &mut cards {
            c.y -= self.scroll;
        }
        cards
    }
    pub fn frame(&mut self, width: f32, height: f32, tab: bool, now: i64) -> Value {
        let show_parameters = self.settings.parameters && !tab;
        let cards = if show_parameters {
            self.cards(width, height)
        } else {
            vec![]
        };
        let bottom = self.view_bottom
            - if self.content_bottom > self.view_bottom {
                8.
            } else {
                0.
            };
        let mut above = 0;
        let mut below = 0;
        let cards=cards.into_iter().map(|c|{let rows=c.rows.iter().enumerate().map(|(i,r)|{let y=c.y+2.+i as f32*8.;if y<22.{above+=1;}else if y+8.>bottom{below+=1;}match r{Row::Header(s)=>json!({"type":"header","text":s}),Row::Subheader(s)=>json!({"type":"subheader","text":s}),Row::Item(p,indent)=>json!({"type":"item","name":p.name,"value":format_value(&p.value),"indent":indent,"alpha":alpha(now.saturating_sub(p.time),i64::from(self.settings.expires)*1000),"color":match p.value{Value::Bool(true)=>0xff55ff55u32,Value::Bool(false)=>0xffff5555,Value::Null=>0xff808080,_=>u32::MAX}})}}).collect::<Vec<_>>();json!({"x":c.x,"y":c.y,"width":c.width,"height":c.height,"rows":rows})}).collect::<Vec<_>>();
        let geometry = if self.settings.geometry {
            self.sections
                .values()
                .flat_map(|s| s.geometry.values().cloned())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        json!({"running":true,"cards":cards,"bottom":bottom,"contentRight":self.content_right,"overflow":show_parameters&&self.content_bottom>self.view_bottom,"above":above,"below":below,"geometry":geometry,"predicted":if self.settings.simulated_player{self.predicted.clone()}else{vec![]},"graph":self.settings.graph.then(||json!({"curve":self.settings.curve,"samples":(0..=1800).map(|i|{let x=i as f32*0.1;[x,crate::tracers::curve::sample(&self.settings.curve,x)]}).collect::<Vec<_>>()}))})
    }
}
pub fn alpha(age: i64, expires: i64) -> f32 {
    if age <= 1000 {
        1.
    } else {
        1. - 0.6 * ((age - 1000) as f32 / (expires - 1000).max(1) as f32).clamp(0., 1.)
    }
}
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(v) => v.to_string(),
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                n.to_string()
            } else {
                let s = format!("{:.3}", n.as_f64().unwrap_or(0.));
                let s = s.trim_end_matches('0').trim_end_matches('.');
                if s == "-0" { "0".into() } else { s.into() }
            }
        }
        Value::String(v) => v.clone(),
        Value::Array(v) => {
            let mut s = format!("[{}]", v.len());
            if !v.is_empty() {
                s.push(' ');
                s.push_str(
                    &v.iter()
                        .take(8)
                        .map(format_value)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                if v.len() > 8 {
                    s.push_str(", …");
                }
            }
            s
        }
        Value::Object(v) => {
            let mut s = format!("[{}]", v.len());
            if !v.is_empty() {
                s.push(' ');
                s.push_str(
                    &v.iter()
                        .take(8)
                        .map(|(k, v)| format!("{k}={}", format_value(v)))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                if v.len() > 8 {
                    s.push_str(", …");
                }
            }
            s
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_filter_expiry_and_change_callbacks() {
        let mut d = Debug::default();
        assert_eq!(
            (
                d.settings.expires,
                d.settings.width,
                d.settings.ticks_to_predict
            ),
            (5, 180, 20)
        );
        d.put("Timer", true, "v", Priority::Normal, json!(1), 10);
        assert!(d.sections.is_empty());
        d.put("Debug/Child", false, "v", Priority::Normal, json!(1), 10);
        d.expire(5009);
        assert_eq!(d.sections["Debug/Child"].parameters.len(), 1);
        d.expire(5010);
        assert!(d.sections["Debug/Child"].parameters.is_empty());
        d.settings.title_filter = Filter::new("^Accepted").unwrap();
        d.filter_changed();
        d.put("Debug", false, "v", Priority::Normal, Value::Null, 0);
        assert!(!d.sections["Debug"].accepted);
        d.put_geometry("Accepted", false, "a", json!({"type":"line"}));
        d.geometry_changed();
        assert!(d.sections["Accepted"].geometry.is_empty());
        d.clear();
        assert!(d.sections.is_empty());
    }
    #[test]
    fn priority_columns_scroll_and_logging() {
        let mut d = Debug::default();
        d.settings.debug_log = true;
        d.enabled();
        for i in 0..30 {
            d.put(
                "Debug",
                false,
                &format!("p{i}"),
                Priority::Low,
                json!(i),
                100,
            );
        }
        d.put("Other", false, "first", Priority::High, json!(true), 100);
        let c = d.cards(500., 140.);
        assert!(matches!(&c[0].rows[0],Row::Header(s)if s=="Other"));
        assert_eq!(c[1].x, 302.);
        assert!(d.scroll(1., [125., 30.], true));
        assert!(!d.scroll(1., [10., 30.], true));
        assert!(!d.scroll(1., [125., 30.], false));
        assert_eq!(d.log.len(), 31);
        assert!(d.log[30].ends_with("\tOther\tfirst\ttrue\n"));
        assert!((alpha(5000, 5000) - 0.4).abs() < 1e-6);
        assert_eq!(format_value(&json!(-0.00001)), "0");
    }

    #[test]
    fn capture_expiry_filter_and_graph_geometry_frame_smoke() {
        let mut d = Debug::default();
        d.settings.graph = true;
        d.settings.simulated_player = true;
        d.predicted = vec![[1., 2., 3.], [2., 3., 4.]];
        d.put(
            "Debug/Client",
            false,
            "Position",
            Priority::Normal,
            json!([1.5, 64., -2.]),
            100,
        );
        let geometry =
            json!({"type":"line","from":[1.,2.,3.],"to":[2.,3.,4.],"color":4278190335u32});
        d.put_geometry("Debug/Client", false, "path", geometry.clone());
        let f = d.frame(600., 400., false, 100);
        assert_eq!(f["cards"][0]["rows"][2]["name"], "Position");
        assert_eq!(f["cards"][0]["rows"][2]["value"], "[3] 1.5, 64, -2");
        assert_eq!(f["geometry"][0], geometry);
        assert_eq!(f["predicted"][1], json!([2., 3., 4.]));
        assert_eq!(f["graph"]["samples"].as_array().unwrap().len(), 1801);
        assert_eq!(f["graph"]["curve"][1], json!([50., 60.]));
        assert!(
            d.frame(600., 400., true, 100)["cards"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        d.expire(5100);
        assert!(
            d.frame(600., 400., false, 5100)["cards"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(d.frame(600., 400., false, 5100)["geometry"][0], geometry);
        d.settings.title_filter = Filter::new("^Other$").unwrap();
        d.filter_changed();
        d.put("Debug", false, "ignored", Priority::High, json!(1), 6000);
        d.put_geometry("Debug", false, "ignored", json!({"type":"point"}));
        let f = d.frame(600., 400., false, 6000);
        assert!(f["cards"].as_array().unwrap().is_empty());
        assert!(f["geometry"].as_array().unwrap().is_empty());
    }
}
