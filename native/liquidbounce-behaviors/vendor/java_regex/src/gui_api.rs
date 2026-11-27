#![no_std]
//! GUICloser's internal boolean-only Java Pattern surface.
//! Derived from java_regex 0.1.0; the uncompiled original API remains in lib.rs.
extern crate alloc;
#[allow(dead_code)]
mod engine;
mod parser;
#[allow(dead_code)]
mod types;
#[path = "jdk_unicode.rs"]
mod unicode;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use engine::{Engine, State};
use parser::Parser;
pub use types::PatternSyntaxError;
use types::*;

#[derive(Debug, Clone)]
pub struct Regex {
    source: String,
    pattern: Pattern,
    flags: Flags,
    group_count: usize,
    named_groups: BTreeMap<String, usize>,
    supplementary: bool,
}
impl Regex {
    pub fn new(pattern: &str) -> Result<Self, PatternSyntaxError> {
        let flags = Flags::default();
        let (parsed, group_count, named_groups) = Parser::new(pattern, flags).parse()?;
        let supplementary = pattern_supplementary(&parsed, flags);
        Ok(Self {
            source: pattern.to_string(),
            pattern: parsed,
            flags,
            group_count,
            named_groups,
            supplementary,
        })
    }
    pub fn pattern(&self) -> &str {
        &self.source
    }
    pub fn uses_supplementary_search(&self) -> bool {
        self.supplementary
    }
    /// Observable compilation metadata used by the independent JDK oracle.
    pub fn lookbehind_uses_supplementary(&self) -> Vec<bool> {
        fn visit(pattern: &Pattern, out: &mut Vec<bool>) {
            for branch in &pattern.branches {
                for node in branch {
                    visit_node(node, out);
                }
            }
        }
        fn visit_node(node: &Node, out: &mut Vec<bool>) {
            match node {
                Node::Lookbehind {
                    supplementary,
                    inner,
                    ..
                } => {
                    out.push(*supplementary);
                    visit(inner, out);
                }
                Node::Group { inner, .. }
                | Node::Lookahead { inner, .. }
                | Node::AtomicGroup { inner }
                | Node::FlagGroup { inner, .. } => visit(inner, out),
                Node::Quantified { inner, .. } | Node::Optional { inner, .. } => {
                    visit_node(inner, out)
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        visit(&self.pattern, &mut out);
        out
    }
    pub fn matches(&self, input: &str) -> bool {
        let units: Vec<u32> = input.encode_utf16().map(u32::from).collect();
        let mut engine = Engine::new(&units, self.flags, self.group_count, &self.named_groups);
        let mut state = State::new(self.group_count);
        engine.match_pattern(
            &self.pattern,
            &[Node::Anchor(AnchorKind::EndOfInput)],
            0,
            &mut state,
        )
    }
    pub fn contains_match(&self, input: &str) -> bool {
        let units: Vec<u32> = input.encode_utf16().map(u32::from).collect();
        let mut engine = Engine::new(&units, self.flags, self.group_count, &self.named_groups);
        let mut state = State::new(self.group_count);
        let mut position = 0;
        while position <= units.len() {
            if engine
                .try_match_at_persistent(&self.pattern, position, &mut state)
                .is_some()
            {
                return true;
            }
            position += if self.supplementary && position < units.len() {
                unicode::codepoint_at(&units, position, units.len()).1
            } else {
                1
            };
        }
        false
    }
}
fn literal_supplementary(cp: u32, flags: Flags) -> bool {
    (0xd800..=0xdfff).contains(&cp)
        || cp > 0xffff
        || flags.case_insensitive
            && flags.unicode_case
            && unicode::simple_upper(cp) != unicode::simple_lower(unicode::simple_upper(cp))
}
fn class_supplementary(class: &CharClass, flags: Flags) -> bool {
    class.negated
        || class.items.iter().any(|item| match item {
            CharClassItem::Single(cp) => literal_supplementary(*cp, flags),
            CharClassItem::Range(first, last) => {
                *last > 0xffff
                    || *first <= 0xdfff && *last >= 0xd800
                    || flags.case_insensitive && flags.unicode_case
            }
            CharClassItem::Predefined(pc) => match pc {
                PredefinedClass::NonDigit
                | PredefinedClass::NonWord
                | PredefinedClass::NonWhitespace
                | PredefinedClass::NonHorizWhitespace
                | PredefinedClass::NonVertWhitespace => true,
                PredefinedClass::Digit | PredefinedClass::Word | PredefinedClass::Whitespace => {
                    flags.unicode_class
                }
                PredefinedClass::HorizWhitespace | PredefinedClass::VertWhitespace => false,
            },
            CharClassItem::UnicodeProperty { name, negated } => {
                *negated
                    || unicode::property_supplementary(
                        name,
                        flags.unicode_class,
                        flags.case_insensitive,
                    )
            }
            CharClassItem::Nested(class) => class_supplementary(class, flags),
            CharClassItem::Intersection(left, right) => {
                class_supplementary(
                    &CharClass {
                        negated: false,
                        items: left.clone(),
                    },
                    flags,
                ) || class_supplementary(
                    &CharClass {
                        negated: false,
                        items: right.clone(),
                    },
                    flags,
                )
            }
        })
}
fn pattern_supplementary(pattern: &Pattern, initial: Flags) -> bool {
    pattern.branches.iter().any(|branch|{
        let mut flags=initial;let mut index=0;
        while index<branch.len() {
            let node=&branch[index];
            if let Node::Literal(high)=node {
                if (0xd800..=0xdbff).contains(high)&&matches!(branch.get(index+1),Some(Node::Literal(low))if (0xdc00..=0xdfff).contains(low)) {
                    index+=2;continue;
                }
            }
            let supplementary=match node {
                Node::Literal(cp)=>literal_supplementary(*cp,flags),
                Node::CharClass(class)=>class_supplementary(class,flags),
                Node::SetFlags(next)=>{flags = *next;false},
                Node::FlagGroup{flags:next,inner}=>pattern_supplementary(inner,*next),
                Node::Group{inner,..}|Node::Lookahead{inner,..}|Node::Lookbehind{inner,..}|Node::AtomicGroup{inner}=>pattern_supplementary(inner,flags),
                Node::Quantified{inner,..}=>pattern_supplementary(&Pattern{branches:vec![vec![*inner.clone()]]},flags),
                Node::Optional{inner,..}=>pattern_supplementary(&Pattern{branches:vec![vec![*inner.clone()]]},flags),
                _=>false,
            };
            if supplementary{return true;}index+=1;
        }false
    })
}
