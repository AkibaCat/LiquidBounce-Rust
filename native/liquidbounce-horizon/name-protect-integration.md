# NameProtect at ae0eb396

Implemented the complete `ModuleNameProtect` settings and game-tick mappings,
including all three independently retained Static/Rainbow color modes,
`Replacement`, enabled-by-default `ObfuscateFriends`, and disabled-by-default
`ObfuscateOthers`. Category is Misc. FILE/INTEROP preserve inactive modes. Like
TargetLock/GUICloser, a future PUBLIC serializer must exclude the whole module
(`doNotIncludeAlways`); this project currently exposes FILE/INTEROP only.

The UTF-16 Aho-Corasick matcher is case-sensitive, matches substrings without
word boundaries and removes overlaps globally by descending length then earlier
start. Other mappings are capped to the first 200 input entries before the
UTF-16 length 2..20 filter. Other names use the original word lists, MD5 of UTF-8
name, a big-endian first-eight-byte seed, Kotlin XorWow Random(Long), bounded
sampling, delimiter bits, Fisher-Yates shuffle and lower-case leet substitutions.
Friend mappings follow FriendManager's UTF-16 ordering, remove blank names before
numbering, preserve an explicit empty alias and default to `Friend 0`, etc.
Precedence is other, then friend, then self. Partial player-list removals retain
old mappings until another rebuild trigger; an empty list clears them. Disable
only gates rendering and retains the snapshot.

The independent native text service publishes immutable matcher snapshots. Font
readers never borrow actor State or re-enter Game/Host, including text drawn from
inside another module callback. A snapshot owns its 512-entry LFU match cache;
the update thread caches, other threads match without that cache. Match plans
do not cache colors. Rainbow uses a live monotonic ten-second hue and the pinned
50/255 alpha (its origin is the native service creation time, not a JVM clock).

Plain String drawing/width replaces once before legacy parsing. Styled sequence
drawing/width resolves legacy first, matches across spans, preserves all fields
of the match's starting style, and overrides only its color. Replacement names
are not recursively protected. Supplementary characters use UTF-16 offsets.
The native Style flag models the original color's local name-protection bypass:
color inheritance carries it; a legacy color override clears it; reset restores
the original style. It never serializes into server components. Friend list now
uses the original bypassed copyable real-name span and normal alias span, default
friend indices, and a suggest-command remove button.

Only Font/Hud draw and width entry points are connected. The original mixin
intentionally does not protect StringSplitter splitLines or headByWidth, so those
continue to split the original input; a longer alias can overflow, as upstream.
World/outline text, chat, tab names and TNT owner text share these entry points.

Root successfully applied `integrate-name-protect.py` and
`update-name-protect-locales.py` before the 185-module batch. Do not reapply the
single-use integrator. Owned native patch fragments feed the usual complete
generator; no upstream or staging file was edited by this subtask.

Validation performed by the owner: complete generator composition, unique two
native helpers, 13 complete Rust sources parsed by rustfmt in temporary files,
word-list byte equality, no split/head protection, no plain-string double
substitution and Simplified/Traditional updater idempotency. No Rust/Kotlin
compiler, game, GPU or full-suite test was run by this subtask.

Prepared focused work (root executes once after current staged core generation):

```powershell
python native/liquidbounce-horizon/collect-name-protect-original.py --out build/name-protect-source-01 --run
python native/liquidbounce-horizon/test-name-protect-focused.py --out build/name-protect-focused-01 --oracle build/name-protect-source-01/oracle.json --run
```

The first command compiles the byte-identical pinned NameGenerator.kt and runs
530 source-oracle inputs with real Kotlin Random/Guava/fastutil and original
resources. The second runs 6 model, 2 username, 3 actual native service and 2
actual config tests, plus one all-vector source differential (14 total). It
uses the real staged horizon-client-core text implementation and omits only the
engine actor/Host boundary; it does not codegen the whole engine. `--filter`
supports failure-only retries. Source/preparation hashes and scope are saved
inside each dedicated build directory. Preparation alone does not compile.

InventoryTracker's unchanged two config tests are separately prepared in
`build/inventory-tracker-config-focused-01`; execute with
`test-inventory-tracker-config.py --out build/inventory-tracker-config-focused-01 --run`.

Pinned source references are `ModuleNameProtect.kt`, `NameProtectMappings.kt`,
`NameGenerator.kt`, `FriendManager.kt`, `RenderConfigurables.kt`,
`ColorAdapter.kt`, `MixinFont.java`, `MixinStringSplitter.java`, and
`CommandFriend.kt` under upstream/LiquidBounce. Kotlin Random bytecode was read
from the local kotlin-stdlib-2.4.21.jar to confirm XorWow warmup, seed halves,
upper-bit sampling and rejection sampling, without invoking a compiler.
