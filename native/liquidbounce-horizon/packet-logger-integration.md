# PacketLogger integration

Implemented against LiquidBounce ae0eb396 and the pinned MC 26.3 Horizon protocol. Already integrated by `integrate-packet-logger.py --apply`; do not apply the non-idempotent integrator again. The official native plus engine Check155-02 passed; the earlier Check155-01 failed only at the separate Offhand configuration string borrow, now corrected.

The model supports the original Blacklist/Whitelist filter, C2SPackets/S2CPackets registry lists, ShowFieldType and nonempty Chat/File output targets. The original running override is preserved: a world is not required, while SelfDestruct still suppresses the module. Final packet observers read the actual packet after ordinary listeners, including cancellation; accepted and silent-send paths avoid duplicate records. Threaded packet listeners queue local chat for the game actor. CSV writes use the real configuration directory and reuse one file per enabled period.

`generate-packet-logger-fields.py` reads the pinned protocol declarations and generates direct typed field accesses for 134 clientbound and 67 serverbound variants. Field types are real Rust `type_name` values, rather than JVM reflection names. Unknown variants retain their actual protocol identifier and raw payload bytes; their undecoded payload is not represented as decoded fields. Both registry REST endpoints expose the actual pinned protocol identifier catalogs.

Rich local chat preserves receive/send/cancel colors and copy actions for IDs, packet names, field names, types and values. CSV escapes commas, newlines and quotes. Simplified and traditional Chinese descriptions and runtime texts follow the loaded game language.

One focused smoke passed: `build/packet-logger-smoke-01.log`. It covers filter directions, nonempty output targets and CSV escaping. Check155-02 is compilation evidence, not live game or full module parity evidence. Full integration tests and a fresh executable remain deferred until the 236-module implementation is complete, as requested by the user.
