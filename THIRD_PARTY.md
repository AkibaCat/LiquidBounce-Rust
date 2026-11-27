# 来源、版权与第三方许可

本工程是 LiquidBounce 的非官方 Rust 移植，工程代码按 GPL-3.0-or-later 分发，见根目录 [LICENSE](LICENSE)。下列第三方内容保留原作者版权与各自许可；根目录 GPL 不会取代第三方许可或 Mojang 的资源授权。

## 主要来源

| 项目 | 固定版本 / revision | 许可与说明 |
| --- | --- | --- |
| [LiquidBounce](https://github.com/CCBlueX/LiquidBounce) | `ae0eb396c07f22383789b28d5912d714c6ca0a86` | Copyright 2015–2026 CCBlueX and contributors；GPL-3.0。模块行为、主题、标识、资源词表与测试夹具中保留其派生内容。 |
| [Horizon](https://github.com/Jijjy/mc-horizon) | `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` | Copyright 2026 Ludovic Marie and the Horizon contributors；[MIT](licenses/Horizon-MIT.txt)。此处固定用户提供的恢复镜像，原项目为 horizon-rust/horizon。 |
| [Horizon-Mixins](https://github.com/WahhaWood/horizon-mixins) | `5a64af938ec86b00d53d16a408aabc0038fb3e57` | Copyright 2026 levzzz；[MIT](licenses/Horizon-Mixins-MIT.txt)。 |
| [LiquidLauncher](https://github.com/CCBlueX/LiquidLauncher) | `a2420d4e93dc2700bfedca68ecbef143dfb66c2b` | CCBlueX and contributors；GPL-3.0。启动器视觉风格与微软 / Xbox 登录流程的参考来源。 |
| [Minecraft-Agent](https://github.com/CCBlueX/Minecraft-Agent) | `0abf69ac35157f33348310326017f4e3fea6b95f` | CCBlueX and contributors；GPL-3.0。导航、模拟与相关差异测试的派生来源，固定记录位于 `research/minecraft-agent-baseline.json`。原 JVM 开发验证产物不随源码分发。 |

前三个上游的完整源码通过 [Restore-Source-Baselines.py](Restore-Source-Baselines.py) 下载并验证，固定哈希位于 [source-baselines.json](source-baselines.json)。上游目录中的原始 LICENSE、NOTICE 和第三方来源记录会随恢复保留。

## 本工程内保留的第三方内容

- `native/liquidbounce-behaviors/vendor/java_regex/`：java_regex 0.1.0，registry revision `ed518dc23dacbe1a88d7cb3f26f0cfe31cc91393`，MIT OR Apache-2.0。完整文本见该目录的 `LICENSE-MIT` 与 `LICENSE-APACHE`；本地修改与逐文件来源见 `PROVENANCE.json`。其 JDK25 观测数据表由本工程生成，不包含 JDK 实现源码。
- `native/liquidbounce-launcher/assets/Inter-*.ttf`：Copyright The Inter Project Authors，SIL Open Font License 1.1，完整文本见 [OFL-Inter.txt](native/liquidbounce-launcher/assets/OFL-Inter.txt)。中文 fallback 在运行时读取本机系统字体，未分发 Windows 中文字体。
- LiquidBounce 派生的启动器与主题素材：GPL-3.0，上游版权与具体使用见 [启动器 NOTICE](native/liquidbounce-launcher/NOTICE.md) 和 [Horizon 适配 NOTICE](native/liquidbounce-horizon/NOTICE.md)。
- 编译所需正弦数值表：`native/liquidbounce-behaviors/src/navigation/simulation/sine-table.bin` 及 `vendor/minecraft-math/` 中的副本用于与固定游戏的数值行为一致；来源与 SHA-256 见该 vendor 目录 README 及采集脚本。它们是源码重建所需的数据输入，不能当作构建缓存删除。

## 构建时获取的依赖

Cargo 版本由各组件的 `Cargo.lock` 固定；npm 依赖锁继承自固定 LiquidBounce 主题。此仓库不把下载缓存、CEF 运行库、CMake 或 Ninja 二进制提交到 Git。恢复后的 Horizon 也包含其自己的第三方许可文件。

`minecraft-auth = 0.1.1` 来自 [CCBlueX/minecraft-auth-rs](https://github.com/CCBlueX/minecraft-auth-rs)，按 LGPL-3.0-or-later 分发；完整许可文本保留于 [minecraft-auth-LGPL-3.0.txt](licenses/minecraft-auth-LGPL-3.0.txt)。启动器通过 Cargo 获取其未经修改的源码。分发链接该库的编译程序时，须保留 LGPL 许可、对应源码与重链接所需信息；便携发布打包流程另行保留该依赖源码。

CEF 使用 `cef` / `cef-dll-sys` 152.3.0 所对应的 CEF 152.0.6，CEF BSD 许可见 [CEF-BSD.txt](licenses/CEF-BSD.txt)。Chromium 及 CEF runtime 内的第三方组件具有各自许可；本地程序打包器从真实 CEF runtime 复制 LICENSE 和 Chromium CREDITS，不能仅用本仓库的 CEF BSD 文本代替这些通知。

Mojang 游戏 JAR、反编译输出、Minecraft 音效、字体、纹理及 prepared Resources 不包含在源码仓库中。用户通过自己的游戏文件和单独资源准备流程提供它们，相关权利仍归原权利人所有。

## 非官方声明

本工程不隶属于 Mojang、Microsoft、Horizon 官方或 CCBlueX。Minecraft、LiquidBounce、LiquidLauncher 及相关标识归各自权利人所有；开源代码许可不表示商标授权或官方认可。
