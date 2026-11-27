# LiquidBounce native adapter for Horizon 9f42064

此目录实现 Minecraft 26.3 的旧版 Horizon 原生 trait-mod 桥接和 `.hnmod` 源码打包。可选 `behavior-hooks` 连接实际游戏阶段与模块逻辑；可选 `webview` 通过真实 CEF 离屏浏览器连接原版 Svelte 主题和客户端线程上的 REST 配置。中文主题从固定原主题生成，随游戏语言切换。全部 236 模块的最新源码与验证分层状态见 `../../research/port-progress.md`；下文五模块报告属于保留的历史证据。

已接入真实的客户端 tick、键盘、鼠标和断开连接事件；从 Horizon 的客户端快照读取真实世界与玩家状态，并调用 `liquidbounce-core` 的输入和生命周期逻辑。键盘经 Horizon 的原版 key name 转为 SDL scan code；鼠标从原生 hook 的 GLFW 顺序转换为 SDL 顺序。默认构建的注册表为空；`behavior-hooks` 只注册已有实际游戏实现的模块，当前数量与证据见进度文档。

Options → Mods 会登记此适配器。默认构建的设置按钮可在关闭菜单并回到已载入的世界后打开诊断屏幕；`webview` 构建则打开真实原版 ClickGUI。Right Shift 也可打开／关闭 ClickGUI。Escape 遵循页面实际设置的 escape layer 状态。

## 带原版浏览器的源码构建

```powershell
python scripts/stage-horizon-build.py --gpu-ui --behavior-hooks --webview
```

staging runner 在窗口／渲染线程、首次 GPU dispatch 前调用 `browser_hooks(BrowserConfig { theme_root, helper_executable, resources_directory, cache_directory, config_directory, initial_size })`，将返回的真实 hooks 传入 `entry::main_with`，退出后调用 `shutdown_browser_current_thread()`。CEF helper、DLL、资源和缓存需按构建脚本部署。

`webview` 构建写出隔离配置目录中的 `interop.json`，其中 `themeUrl` 是实际认证主题 URL。HTTP worker 将操作提交队列，真实客户端 tick 执行原版模块开关、配置 setter 和生命周期。成功的配置保存发生在 HTTP 成功响应之前；保存失败返回 500，并保留已经发生的真实游戏状态。模块状态、隐藏标志、键绑定、设置保存在隔离的 `modules.json`，未实现模块不会作为可用功能列出。NoBob／NoHurtCam 仅有共同 Enabled／Bind／Hidden 设置，与固定原版一致。

原版主题所需的 theme、virtualScreen、localStorage、呈现设置以及真实窗口信息已接线。共享 gameplay global 根目前为空；不存在的全局系统不会显示为已实现选项。Windows CEF VK/native scan code 与向原主题发送的 SDL key code 分开处理，屏幕事件包含原主题需要的 class/title 数据。官方 Horizon 直接控制按键单独通过 `client_control_key` 接入相同 core 输入路径，避免真实 Winit 键被重复执行。

对外模块列表遵循原版 `ModuleManager` 的名称忽略大小写排序；实际模块 ID 与生命周期顺序独立保留，新增模块也使用同一排序规则。

`/api/v1/client/integration-status` 是此移植新增的只读诊断接口，返回实际 world/player gate、模块 running、已执行回调计数、相机／gamma 接入点最近观察到的输入和输出，以及真实 CEF paint/load/console 状态。NoBob／NoHurtCam 还报告 hurt/view 分支、实际 hurt/death/walk/bob 输入和原始／应用／shake／组合矩阵；nausea／portal spin 独立记录。配置值不作为已产生游戏效果的证明。

生产配置代码可独立测试：

```powershell
cargo +stable test --manifest-path native/liquidbounce-horizon/config-tests/Cargo.toml --offline
cargo +stable test --manifest-path native/liquidbounce-horizon/behavior-tests/Cargo.toml --offline
```

9 项配置测试通过，覆盖原版树结构、声明顺序、数值转换、模式切换请求、错误继续处理、Float 等值和非有限输出。镜头测试直接编译固定 Horizon 的真实相机数学源码，6 项新增镜头／生命周期测试及 10 项原测试通过。

五模块源码整合构建与 `build/game-runs/native-integration-09/native-integration-validation.json` 的 58 项完整实机自动检查通过。覆盖真实 FOV、gamma／Night Vision、Zoom 按键释放、非零走路／伤害镜头取消与恢复、分支隔离、死亡 roll 取消且保留死亡 FOV，以及 nausea spin 保留。原版 ClickGUI 的五行名称和顺序另经同目录 `original-clickgui.png`／`ui-visual-review.json` 人工验收通过；客户端正常退出，无该次游戏或 CEF 残留。前三模块的 31 项基线保留在 `native-integration-01/`。非零 native shake 尚未实机产生，其保留路径只有源码与矩阵测试证据。原版 Java 与 Rust 客户端的差分运行尚未执行。实际构建和游戏效果以对应源码／二进制摘要绑定的报告为准。

## 打包

使用 Python 3.11 或更新版本：

```powershell
python .\native\liquidbounce-horizon\package.py
python .\native\liquidbounce-horizon\package.py --validate .\native\liquidbounce-horizon\dist\liquidbounce_horizon-0.1.0.hnmod
```

脚本重新检查恢复源码 ZIP 的完整 Git SHA、SHA-256、游戏版本、415 hooks 和 launcher's mixins pin，将兄弟目录 `liquidbounce-core` 的 Cargo manifest 与 Rust 源码作为 `[[crate]] core` 放入包内，验证路径与格式。输出为：

- `dist/liquidbounce_horizon-0.1.0.hnmod`：原生源码包，需要由对应 launcher 编译并链接游戏。
- `dist/unpacked/`：对应解包目录，可交给该版本的 `native-build`。
- `dist/package-validation.json`：包结构和摘要验证结果。

`engine/Cargo.toml` 的 `horizon-client` registry 占位依赖由旧版 launcher 改为 staged game 的路径；`../core` 是其支持的 sibling crate 路径。不能把此目录当作较新 signed-SDK/dylib 模组加载。完整游戏构建需要 launcher 的匹配 Rust/linker 工具链和玩家自己准备的 Minecraft 26.3 资源。

原版主题资源在项目的 `artifacts/liquidbounce-original-ui.zip`。完整浏览器构建使用上文 `webview` staging 流程；旧版纯 trait `.hnmod` launcher 不会自动应用本项目的 GPU／phase 源码补丁和 CEF 部署，因此该包明确不包含 `webview` feature 的路径依赖，不声称提供已连接的 Chromium。GPU UI 和行为 hook 均是针对固定源码的可选扩展，适配器不假装它们已存在于原始 9f42064。

项目根的 `scripts/Package-LiquidBounce-Preview.ps1` 另行生成完整开发预览，核对实际构建源码、依赖锁、可执行文件与实机报告的 SHA-256。其 `source/` 提供本地移植源码、独立测试、补丁、构建脚本、三份固定上游源码归档及 `README-REBUILD.md`／`Rebuild-LiquidBounce.ps1`，按完整提交和摘要恢复上游源码；重建不依赖原工作区，仍需说明中列出的编译工具与依赖下载。不分发 Minecraft JAR 或游戏资源。该预览的源码重建流程与这里的纯 trait `.hnmod` 包范围不同。

打包验证只验证来源与包结构，不证明游戏已构建或运行，也不证明界面或作弊模块具有原版行为。
