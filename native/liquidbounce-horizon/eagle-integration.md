# Eagle

Fixed source: LiquidBounce ae0eb396, ModuleEagle.kt, isCloseToEdge and ScaffoldBlockItemSelection.isValidBlock.

All seven conditions, conditional pitch range, physical sneak capture/release, flying guard and randomized edge distance are implemented. The real native input phase executes after ordinary movement input listeners and before SafeWalk's objection phase, matching SAFETY_FEATURE ordering. It uses the existing scoped simulated player, next velocity fallback, contextual block and entity collisions and two-tick fall check. HoldingBlocks inspects real main/offhand BlockItem defaults, the full UP collision face, falling block class and the source default disallowed blocks. The shared Scaffold disallowed setting can replace this default when Scaffold is ported.

The native and engine copies use the same Eagle model. File/Interop settings, source aliases, and simplified/traditional labels/help are wired. A focused state/condition test runs once; whole-game and differential verification are reserved for the final pass. Scaffold's editable shared disallowed set remains a future dependency until that module is integrated.
