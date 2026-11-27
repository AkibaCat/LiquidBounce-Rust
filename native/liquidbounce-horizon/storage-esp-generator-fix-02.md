StorageESP freeze revision 02

The official Check159-01 stopped before native compilation because horizon-mod-api-gen scans native hook signatures exclusively in crates/horizon-client/src/game/native_hooks.rs (generator lib.rs:754). The previous hook lived in a sibling storage_esp Game impl. Move only the dispatch signature into native_hooks.rs through storage-esp-patch.py; keep frame/snapshot behavior in storage_esp.rs. This fixes catalog discovery without repeating successful config/geometry/GPU checks.

Parent: regenerate and stage the native patch, then rerun the failed official check. Historical storage-esp-freeze.json remains intact.
