# GUICloser's internal regex implementation

This directory starts from the MIT/Apache-2.0 `java_regex` 0.1.0 registry
release. Its original README, specification and compatibility claims describe
that upstream project. They are not evidence for the local port. The original
uncompiled `src/lib.rs` is retained for provenance; Cargo builds `src/gui_api.rs`.

The local API only compiles pattern strings and evaluates whole-title matches
or the first contained match. It has no replacement, capture-result, Java
Matcher region or streaming API. GUICloser supplies normal Rust strings;
matching runs on their UTF-16 units. Escaped isolated-surrogate pattern values
are retained as numeric code points rather than replaced with U+0000 or U+FFFD.

Local corrections cover JDK property resolution and case mappings, character
names, grapheme boundaries, Unicode and ASCII word boundaries, surrogate-aware
consumption, Start/StartS search selection, Behind/BehindS lookbehind selection,
backreferences, lexical flags and the distinction between optional and counted
quantifiers. The runtime does not start or call a JVM.

The JDK data files contain observations of the pinned JDK 25.0.5.1 runtime's
public Character APIs and reflected predicate/grapheme tables. The generation
program is `native/liquidbounce-horizon/gui-closer-source-fixture/JdkRegexFacts.java`;
`generate-gui-closer-jdk-data.py` stores command logs, generation results, sizes
and SHA256 hashes in a fresh numbered evidence folder. No JDK source is copied
into this implementation.

The upstream five-million-step and 500-frame limits were removed. They returned
ordinary false matches and could change GUICloser's cancellation/printing
behavior. Linear prefixes and scalar repetitions use iteration. Recursive
parsing and matching grow their native stack with pinned `stacker` 0.1.25.

`test-gui-closer-source.py` copies the original LiquidBounce ModuleGUICloser.kt
byte for byte, executes it with actual JDK/Kotlin Regex, and compares both
modes to a copied Rust implementation. It also checks reflected search and
lookbehind compilation metadata and runs Clippy. Numbered failed snapshots are
retained. A finite differential suite is evidence for the cases it contains;
it does not prove every Java Pattern program equivalent. Source tests are
separate from config-serializer, native seam and actual game verification.

This package is internal to the port. Original crate licenses remain in
LICENSE-MIT and LICENSE-APACHE. Local source changes use those same licenses;
the surrounding LiquidBounce-derived module and test harness retain their
GPL-3.0-or-later notices.
