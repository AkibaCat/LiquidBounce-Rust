# LiquidBounce Rust egui 启动器

固定 `300 × 450` 的 Rust + egui 原生启动器。布局依据用户提供的参考图，界面使用 LiquidLauncher 的黑色玻璃面板、圆角控件、白色文字与蓝色 `#4677FF` 强调色；LiquidBounce 标志显示在标题栏。

## 使用

将发布包放在可写目录，双击 `LiquidBounce-Launcher.exe` 后点击“启动游戏”。点击玩家名称打开账户弹窗，可选择微软正版登录或离线登录。启动器直接启动包内 `runtime/liquidbounce-native-game.exe`，默认从 `resources/26.3/` 读取已准备的 Minecraft 26.3 资源；资源设置中仍可选择其他目录。

启动器和游戏的可变配置、世界、日志及浏览器缓存保存在包内 `user-data/`。游戏功能范围沿用发布包的说明：236 个模块已注册，其中 210 个具备 native 逻辑；启动器没有改变模块完成情况。

微软正版登录采用固定 LiquidLauncher 同源的设备授权流程，使用 `minecraft-auth 0.1.1` 和 Microsoft title ID `00000000402b5328`。账户本人需在微软页面确认授权。账户缓存由 Windows DPAPI 加密，游戏会话仅通过子进程标准输入传递；本次构建尚未完成人工微软联网登录验收。

## 构建与打包

在工程根目录运行：

```powershell
cargo +stable build --release --manifest-path .\native\liquidbounce-launcher\Cargo.toml
pwsh -NoProfile -File .\scripts\Package-LiquidBounce-Launcher.ps1 -LauncherExe .\native\liquidbounce-launcher\target\release\liquidbounce-launcher-egui.exe
```

如果设置了 `CARGO_TARGET_DIR`，将打包命令的 `-LauncherExe` 改为实际产物路径。打包器把可执行文件命名为 `LiquidBounce-Launcher.exe` 放在发布包根目录，同时加入 resources、源码与许可证，并在 `ReleasesPack` 创建 `Start-LiquidBounce.cmd` 快捷入口。重建发布包中的启动器时，在本目录运行 `cargo +stable build --release`。

## 界面与字体来源

- LiquidLauncher 样式参照固定提交 `a2420d4e93dc2700bfedca68ecbef143dfb66c2b`，来源 `CCBlueX/LiquidLauncher`，许可证为 GPL-3.0。
- LiquidBounce 标志及主题资产来自本工程固定的 LiquidBounce 提交 `ae0eb396c07f22383789b28d5912d714c6ca0a86`，许可证为 GPL-3.0。
- Inter 字体随程序提供，采用 SIL Open Font License 1.1，完整许可见 `assets/OFL-Inter.txt`；打包时另复制到 `licenses/Inter-OFL.txt`。
- 中文后备字体从本机 Windows 字体目录读取，不随启动器分发。头像依据用户提供的参考图。
- `minecraft-auth 0.1.1` 采用 LGPL-3.0-or-later，发布包附有其源码和 `licenses/minecraft-auth-LGPL-3.0.txt` 完整许可。

详细来源见 `NOTICE.md` 与发布包 `SOURCES.md`。
