# Launcher sources and assets

The native launcher is part of the LiquidBounce / Horizon Rust port and is
licensed under GPL-3.0-or-later, as declared by its Cargo package.

Its compact layout follows the launcher reference image supplied by the user.
The black glass surfaces, rounded controls and #4677FF blue accent reference
LiquidLauncher at commit a2420d4e93dc2700bfedca68ecbef143dfb66c2b:

https://github.com/CCBlueX/LiquidLauncher/tree/a2420d4e93dc2700bfedca68ecbef143dfb66c2b

LiquidLauncher is copyright CCBlueX and contributors and is licensed under
GPL-3.0. The LiquidBounce mark and theme assets are derived from LiquidBounce
at commit ae0eb396c07f22383789b28d5912d714c6ca0a86, also GPL-3.0:

https://github.com/CCBlueX/LiquidBounce/tree/ae0eb396c07f22383789b28d5912d714c6ca0a86

The bundled Inter fonts are copyright The Inter Project Authors and are
licensed under the SIL Open Font License 1.1. The full font license is included
in assets/OFL-Inter.txt. The font files match the corresponding pinned upstream
assets; they have not been modified by this launcher project.

Chinese fallback fonts are read from the local Windows fonts directory at
runtime. No Windows Chinese font file is redistributed with this launcher.
The avatar is based on the user-provided launcher reference image.

Microsoft/Xbox/Minecraft authentication follows the pinned LiquidLauncher
implementation and uses minecraft-auth 0.1.1 (LGPL-3.0-or-later):

https://github.com/CCBlueX/minecraft-auth-rs

The Microsoft title ID is 00000000402b5328, as selected by the upstream
dependency. It is separate from the auth.liquidbounce.net application ID.
The release includes the unmodified crate source under
source/upstream/minecraft-auth/minecraft-auth-0.1.1/ and its full license at
licenses/minecraft-auth-LGPL-3.0.txt. Windows DPAPI protects the local account
cache; the game session is transferred through the child process's stdin.
Interactive Microsoft authorization requires the account owner. Manual online
Microsoft login has not been verified for this build.
