# BookBot

Pinned LiquidBounce ae0eb396 `ModuleBookBot.kt`, actual category Exploit.
Uses the shared InventoryManager, all 41 player slots in source Slots.All order,
captured selected-slot swap guard, native font advance, real WRITTEN_BOOK_CONTENT
and EditBook packet. Random/File generation, both page counts, Sign/Name/Delay,
complete inventory constraints, enable reset and disable-on-quit are wired.
Simplified/traditional Chinese labels and setting descriptions follow game language.

One content smoke passed (`build/book-bot-smoke-01.exe`), covering production
pagination and UTF-16 narrowing. Official 114 Check passed; no cumulative tests
or game run were repeated. Full in-game and upstream differential validation pending.

Limits recorded for final validation: file decoding currently accepts UTF-8 text;
Java CodePointReader malformed-input and OS-dependent behavior are not compared.
A nonempty cyclic source containing only discarded CR/initial-space characters
is rejected to avoid an unbounded game-thread loop. Empty streams still write an
empty page list, as in the source. Unpaired UTF-16 conversion uses '?'; actual
Netty byte comparison is pending. No future inventory module is represented by a stub.
