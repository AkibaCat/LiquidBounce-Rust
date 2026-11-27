# LiquidBounce Rust · Minecraft 26.3

An unofficial Rust port of LiquidBounce for Minecraft 26.3, powered by Horizon.

基于 Horizon 的 LiquidBounce Rust 移植工程，包含原生游戏接口、模块行为、CEF 网页界面和 Rust / egui 启动器。主要开发与验证平台为 Windows x64。

**这是仍在开发的非官方移植版。236 个上游注册模块中，当前 210 个已接入原生行为，26 个仍未完成。** “已接入”表示代码路径已实现，不代表全部配置组合、服务器行为和视觉效果都与上游等价，也不代表已经通过 236 项完整验收。当前状态以 [模块清单](research/module-parity.json)、[进度记录](research/port-progress.md) 和实际测试证据为准。

## 已有内容

- Rust 模块注册、配置、事件与原生 Horizon 行为适配。
- 基于 LiquidBounce 原主题的 CEF 界面；主菜单、单人世界和服务器列表连接真实游戏数据及操作。
- 界面与模块配置的中文显示；游戏语言为简体中文或繁体中文时使用对应翻译，其他语言保留英文。
- 300 × 450 逻辑像素的 egui 启动器，使用 LiquidBounce 风格背景和控件。点击名字可选择微软正版登录或离线登录。
- 微软设备码登录沿用 LiquidLauncher 的 Xbox title ID `00000000402b5328`。Windows 使用 DPAPI 保护账号缓存，游戏会话通过子进程标准输入传递。**真人微软账号授权与在线服务器登录仍需人工验收。** 离线账号用于离线游戏和允许离线账号的服务器。
- 独立资源准备、便携资源校验和发布包生成脚本。

菜单接入目前覆盖主菜单、单人世界列表和多人服务器列表的核心路由，以及世界启动、原生创建/编辑/删除确认、服务器列表的增删改与连接。Proxy Manager、游戏内账户切换、Basic Mode、Toggle Shader / 背景切换和部分社交 `openLink` 尚未完整接入后端；这些入口不能视为已完成。三页菜单的最终构建及运行验收以对应测试报告为准。

## 源码结构

| 目录 | 内容 |
| --- | --- |
| `native/liquidbounce-core` | 模块目录、公共配置与注册信息 |
| `native/liquidbounce-behaviors` | Rust 行为实现、共享算法与编译数据表 |
| `native/liquidbounce-horizon` | 原生 Host 适配、补丁生成器、Rust 模板和测试夹具 |
| `native/liquidbounce-interop` | 网页 REST / WebSocket 协议与本地化 |
| `native/liquidbounce-webview` | CEF 离屏渲染与 Horizon GPU 合成 |
| `native/liquidbounce-theme` | 原主题上的本地化覆盖源码 |
| `native/liquidbounce-launcher` | egui 启动器及账号登录 |
| `patches` | 对固定上游源码应用的补丁 |
| `scripts` | 构建、资源准备、验证与打包工具 |
| `research` | 上游版本固定记录、模块清单、来源与差异说明 |

`upstream/`、`build/`、资源和编译产物不提交到 Git。请编辑本工程的 Rust 源码、补丁与生成器；构建脚本在 `build/staging/` 中应用改动，保留恢复后的上游基线。

## 恢复固定上游源码

需要 Python 3.11 或更新版本。首次执行会下载三个固定源码归档，并验证 SHA-256、ZIP 中的完整 Git commit、CRC 与解压路径；已有上游文件如与归档不同，会保留原文件并报错。

```powershell
python Restore-Source-Baselines.py
python Restore-Source-Baselines.py --verify-only
```

离线环境可复用另一份工程的 `research/` 归档缓存：

```powershell
python Restore-Source-Baselines.py --offline --archive-cache D:/SourceArchiveCache
```

| 上游 | 固定 revision | 许可证 |
| --- | --- | --- |
| [Horizon 恢复镜像](https://github.com/Jijjy/mc-horizon) | `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` | MIT |
| [LiquidBounce](https://github.com/CCBlueX/LiquidBounce) | `ae0eb396c07f22383789b28d5912d714c6ca0a86` | GPL-3.0 |
| [Horizon-Mixins](https://github.com/WahhaWood/horizon-mixins) | `5a64af938ec86b00d53d16a408aabc0038fb3e57` | MIT |

完整归档地址和哈希见 [source-baselines.json](source-baselines.json)。`research/Horizon-source.zip` 是 staging 必需输入，恢复后请保留。微软登录的参考实现固定为 LiquidLauncher `a2420d4e93dc2700bfedca68ecbef143dfb66c2b`。

## 构建

Windows x64 需要：Rust stable 1.95 或更新版本、MSVC x64 C++ 工具与 Windows SDK、Git、Python 3.11+、PowerShell 7、Node.js 22.15+ 和 npm。初次构建会下载 Cargo / npm 依赖与固定版本 CEF；游戏资源准备另外需要 Java 25 和对应 Horizon 资源准备工具。

先构建本地化主题，再构建游戏。脚本将源码放入独立 staging、生成 native API，并准备 CEF 152.0.6：

```powershell
python scripts/build-localized-theme.py --name chinese --build
pwsh -File scripts/Build-LiquidBounce.ps1
pwsh -File scripts/Build-LiquidBounce-Launcher.ps1
```

主要输出：

```text
build/themes/chinese/dist/
build/staging/horizon/target/debug/liquidbounce-native-game.exe
native/liquidbounce-launcher/target/release/LiquidBounce-Launcher.exe
```

游戏构建驱动默认生成 debug 程序，启动器生成 release 程序。游戏采用本工程的 native runner，不是可直接放入 Fabric `mods/` 的 JAR。CEF runtime、helper、主题和游戏资源都需要一并配置或打包，单独一个游戏 EXE 不构成完整安装。

## 单独准备游戏 Resources

源码仓库不包含 Mojang 游戏 JAR、音效、字体、世界、账号、完整游戏资源或本地生成的资源缓存。资源准备使用用户已有的正版 Minecraft 26.3 文件，并按 Horizon 固定的游戏哈希验证。准备阶段需要 Java；原生游戏运行不依赖 JVM。

先提供匹配固定 Horizon revision 的资源准备工具目录、Java 25、26.3 的版本 JSON，以及 Gradle / Fabric Loom 缓存中的 client/server JAR：

```powershell
python scripts/prepare-local-resources.py --release D:/HorizonTools --java-home D:/Java/jdk-25 --version-json D:/Minecraft/26.3.json --cache "$env:USERPROFILE/.gradle/caches"
python scripts/prepare-portable-assets.py --prepared build/resources/26.3 --output build/portable-resources/26.3 --download
```

`--release` 目录须含 `horizon-prepare.exe`、`research-lock.json`、`exporters/`、`tools/` 与 `mods/`，且版本锁与已恢复的 Horizon 基线一致。资源准备脚本仍保留开发机默认路径，其他机器应显式传入这些参数。`--download` 仅用于补齐经哈希验证的官方资源，不会获取游戏账号或复制个人世界。详细说明见 [便携资源说明](scripts/PORTABLE-RESOURCES.zh-CN.md)。

## 验证与打包

修改若干模块后统一运行一次构建与对应测试。基础编译检查与引擎单元测试：

```powershell
pwsh -File scripts/Build-LiquidBounce.ps1 -Check
pwsh -File scripts/Build-LiquidBounce.ps1 -TestEngine
python scripts/test-stage-horizon-patches.py
```

模块独立测试位于 `native/liquidbounce-horizon/test-*.py`，Rust 测试位于各 crate 的源码或 `tests/`。其中部分历史 JVM 差异测试需要额外的 JDK / Kotlin / Minecraft-Agent 开发环境和用户自己的游戏字节码，不能在干净克隆后直接全部运行；这些输入和开发日志不会作为源码分发。引擎重入锁回归测试覆盖首次进世界时的输入回调死锁路径。

有匹配的 Horizon 控制器和本地资源后，可进行隔离世界测试：

```powershell
pwsh -File scripts/Start-LiquidBounce-Test.ps1 -Name qa -Controller D:/HorizonTools/horizon-ctl.exe
python scripts/verify-native-integration.py --run build/game-runs/qa --controller D:/HorizonTools/horizon-ctl.exe --languages --quit
```

运行时验证脚本覆盖其已实现的场景，不代表验证了全部 236 个模块。界面截图需人工检查；真人微软授权需由账号所有者操作。测试报告与日志均写入忽略的 `build/`。

对同一游戏程序 SHA-256 运行主菜单 API、进出世界和退出生命周期的离屏烟测，见 [原生启动和菜单烟测说明](scripts/qa/README.zh-CN.md)。这些脚本显式接收程序哈希、CEF / 主题包和 Resources 路径，并把结果留在独立测试目录中。

2026 年 11 月 27 日的最终构建已使用同一个程序 SHA-256 通过离屏基础烟测：三页菜单路由的真实 DOM ACK 与后续新 OnPaint、`servers.dat` 增删改和排序、全部 236 项模块配置的实际保存、进入世界后保存返回主菜单及正常退出。三项针对性引擎回归测试同时通过，覆盖输入回调锁重入、配置序列化与菜单输入接管。以上结果属于启动与菜单流程验收，完整模块功能和交互视觉仍须继续验证。

预览包生成要求实际通过的完整集成报告和截图验收记录；报告不满足条件时打包器会拒绝：

```powershell
pwsh -File scripts/Package-LiquidBounce-Preview.ps1 -Name local-preview -ValidationReport build/game-runs/qa/native-integration-validation.json
```

得到游戏预览目录后，再把启动器和便携 Resources 加入该目录，或放入本地 `ReleasesPack` 下的同一包目录：

```powershell
pwsh -File scripts/Package-LiquidBounce-Launcher.ps1 -LauncherExe native/liquidbounce-launcher/target/release/LiquidBounce-Launcher.exe -Resources build/portable-resources/26.3 -Destination artifacts/local-preview
```

启动器读取包内的游戏程序、CEF、主题和 `resources/26.3`。发布包的个人配置与账号缓存放在 `user-data/`；发布源码不包含这些数据。`refresh-liquidbounce-preview.py` 提供已验证游戏程序的就地更新，`sync-portable-resources.py` 提供资源同步计划与应用，使用前请阅读各自的 `--help`。

## 许可证与来源

本移植工程按 **GPL-3.0-or-later** 分发，完整文本见 [LICENSE](LICENSE)。上游版权、MIT / Apache、LGPL、字体许可证及固定来源记录见 [THIRD_PARTY.md](THIRD_PARTY.md) 和各组件附带的 LICENSE / NOTICE。分发编译程序时须同时遵守其依赖和资源的许可证；Mojang 资源不因本工程使用 GPL 而改变授权。

本项目与 Mojang、Microsoft、Horizon 官方及 CCBlueX 没有隶属或官方合作关系。Minecraft、LiquidBounce 和 LiquidLauncher 名称及标识归各自权利人所有。
