# 移植核对记录

## 固定原版功能范围

`ModuleManager.registerInbuilt()` 实际注册 236 个模块，按原顺序完整记录在 `module-parity.json`。该目录有 696 个 Kotlin 文件、239 个直接 `ClientModule` 对象；`ModuleDroneControl`、`ModuleProjectileAimbot`、`ModuleBetterTitle` 不在原版 builtin 数组内。不能将模式、辅助文件或未注册对象计为已移植功能。

最新源码与验证分层状态见 `port-progress.md`。下文保留五模块阶段的历史实机证据及其精确范围，不代表当前全部源码已经通过同等验收。`module-parity.json` 是 236 模块的源码与证据索引；注册数量、翻译数量和独立测试均不能替代实际游戏中的完整行为验证，未实现模块不向界面提供工作开关。

## 原版界面接入

原版完整前端在 `upstream/LiquidBounce/src-theme`，使用 Svelte 5 / Vite 6，包含 ClickGUI、HUD、菜单、Inter 字体、图标和背景。生产构建已通过；对应 Web 资源包包含原许可证。

| 原版能力 | 源码依据 | Horizon 端需要实现 |
| --- | --- | --- |
| 页面路由与虚拟屏幕 | `src-theme/src/App.svelte` | 页面显隐、游戏屏幕状态与 UI 页面同步 |
| REST 服务 | `src-theme/src/integration/rest.ts` | 原 `/api/v1/*` 路径、数据结构、配置与错误行为 |
| WebSocket | `src-theme/src/integration/ws.ts`、`events.ts` | 根路径 WS、`{name,event}`、订阅与状态更新 |
| 本地认证 | `integration/interop/middleware/AuthPlugin.kt` | `lb_code` 初始化与 `lb_auth` cookie |
| 浏览器渲染 | `integration/backend/BrowserBackend.kt`、`BrowserTexture.kt` | 离屏 Web 渲染、纹理格式、合成、DPI 和 viewport |
| 输入 | `integration/backend/input/InputHandler.kt` | 键码、扫描码、修饰键、Unicode、鼠标、滚轮 |
| 原生 HUD | `integration/theme/component/HudComponentFactory.kt` | Web 与游戏原生 HUD 组件共同渲染 |

CEF 由独立的 <https://github.com/CCBlueX/LiquidBounce-Addon-CEF> 提供，主仓库依赖版本 `1.1.0+26.3`。主仓库的 Web 主题完整，不等于已经取得或完成移植全部 Chromium / JCEF / 原生 GPU 实现。

## 游戏逻辑接入

LiquidBounce 的 185 个客户端 Mixin 直接操作 Java/Fabric 的玩家、世界、包、输入及 Mojang GPU 对象。移植须保持 127 个事件的相同时点、可变字段、取消、优先级、协程等待与退出取消语义，并迁移共享的旋转、玩家预测、库存和目标追踪系统。

| 所需原版语义 | 缓存 Horizon 资料目前确认的能力 | 需要进一步核对 |
| --- | --- | --- |
| 本地玩家 tick / movement 可取消；运动包 PRE/POST | `client_tick` 为不可取消客户端 tick；`player_tick_pre` / `player_move` 为服务器侧 | 精确旧版 SDK 的本地玩家和运动发包插入点 |
| 远程连接的客户端收发包取消 / 改写 / 延迟 | `server_packet` 描述的是服务器发包 | 客户端收包、发包、队列与线程语义 |
| 目标 / 实体 / 世界读写与碰撞 | 有实体状态、运动及区块快照碎片 | 完整类型、可变访问、射线、物理与生命周期 |
| 画布 / ESP / GPU 管线 | 有世界几何画布；公共画布说明不暴露 GPU 对象 | 离屏网页任意纹理上传、合成与自定义渲染路径 |

这些能力描述来自参考缓存，未确认的能力不等于 Horizon 完全不支持。尤其不能把服务器侧 hook 用作远程服务器中的客户端 hook，也不能将新版分叉重构后的 API 当作本地 `9f42064` ABI。

## 源码恢复与精确旧版构建

原仓库 GitHub API 返回 404。存活分叉 `arismoko/horizon` 的 GitHub 元数据中 parent/source 均为原仓库。它和用户提供的 `Jijjy/mc-horizon` 镜像都保留 `9f42064`。已从用户的镜像完整下载 53,701,917 字节归档，ZIP comment 确认完整提交 `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`。归档 SHA-256 为 `4a9798eaf7b5dce1f711307efe638f4f9eed8eafc3d58cb0415ef63a38f1c95e`；6448 个解压文件全部与 CRC 验证后的归档内容一致。

原参考缓存没有完整 workspace 或 SDK。恢复后的旧版源码已确认：`horizon-mod-api-gen` 在构建时从真实源码与 `native-hooks.json` 生成 typed trait；原生模组 API 直接依赖 `horizon-client`，与新版 SDK facade 方案不同。纯 trait 包也要生成 `horizon-native-game` 并编译客户端，不是把 DLL 放入模组目录即可加载。

旧版包格式是 `horizon-native.json`（schema 1，完整 Horizon / mixins 提交号）与 `mixins/<id>/horizon-mixin.toml`。纯 trait 包使用 `[api] path = "engine"`，导出 `pub fn hooks() -> Box<dyn horizon_client::mod_api::Hooks>`；其共享 crate 必须显式声明为 `[[crate]]`。包中有 `[[injection]]`、状态、桥等源码修改时还需调用 `horizon-mixins`，然后重新编译。

固定注入器提交为 `5a64af938ec86b00d53d16a408aabc0038fb3e57`。原注入器仓库也返回 404，已从 `WahhaWood/horizon-mixins` 镜像恢复该提交至 `upstream/Horizon-Mixins`。不使用其它 mirror 的新接口替代固定版本。

## 已实现基础及仍需实现的能力

`native/liquidbounce-core` 已通过独立 Rust 行为测试：可变事件分派、优先级／稳定顺序／快照、取消与完成状态、世界和玩家 gate、模块生命周期、同步回调／重入／异常顺序及 TOGGLE/HOLD/SMART 键绑定。它不含具体战斗、运动或网络模块；共享配置、协程及各模块相关管理器仍须按实际需求移植。

精确旧版源码进一步确认：`native_client_packet` 是 game thread 的只读通知，不覆盖网络线程自行处理的 keepalive 等包；`native_send_packet` 是发送入口，不是拦截原版出包。`remote.rs::handle_packet`、`Game::send`、`net.rs::receive_play` 及 `remote_send_state` 需要原版相同时点的事件接入，才能实现取消／延迟／改写和运动包 PRE/POST。

Horizon 自有 `Screen::Mod` 能提供世界中的 mod 屏幕与输入捕获；原公共 mod 图像槽只有共享的 1024×1024 HUD atlas。`patches/horizon-ui-render-hook.patch` 新增最终 HUD/GUI 后、HDR 编码前的独立 GPU 图层接口；完整原版源码上 `git apply --check` 已通过。真实 CEF OSR、任意尺寸 BGRA 纹理上传和原版 REST/WS bridge 已实现。CEF fixture、未改动原主题的独立加载，以及五模块客户端中的真实文档加载、绘制、ClickGUI 开闭与可见模块行均已通过对应报告；所有 UI 页面、DPI 和原生 HUD 尚未逐项完成对照。

NoBob／NoHurtCam 使用 `native/liquidbounce-horizon/patches/horizon-behavior-hooks.patch` 的 `camera_bob_effects` 在真实 `Game::bob_matrix` 中分别取消走路 bob 和 hurt/death roll；仅修改本次计算的局部输入，保留 avatar 历史、游戏选项、原 native shake 组合和外层 nausea／portal spin。`cameraBobResult` 与 `cameraSpinningResult` 观察实际输入及返回矩阵，供官方接口验收。Sprint 已接入实际行为。AutoWalk、AntiAFK、CustomAmbience 仍只完成依赖分析，未注册为实际模块，详见 `next-render-modules.md`。

## 行为一致的验收

原版与 Rust 版本须使用相同初始世界、玩家 / 实体、输入序列、随机种子和 tick 序列，比较模块开关及配置结果、目标 / 旋转 / 移动、收发包内容与顺序、取消和协程恢复。界面则需比较真实游戏内的布局、DPI、输入、配置同步与原生 HUD。原版构建成功或单纯展示原版网页均不能代替这些验证。

已对 Zoom、NoFOV、FullBright 的纯逻辑运行 18 项行为测试，包括独立 Java 数值向量、真实 26.3 Mth 字节码核对与生命周期边界。Rust core 的同步回调／重入／异常顺序及原输入行为共 22 项测试通过；原版 HTTP auth／config 请求、真实 TCP 模块操作与 WebSocket envelope、UI 持久化共 7 项测试通过。生产配置解析与序列化有 9 项测试；新建 `native/liquidbounce-horizon/behavior-tests` 直接编译固定 Horizon 相机数学源码，通过 6 项新增镜头／生命周期测试及其 10 项原测试。测试含独立 hurt/walk 取消、死亡 roll 与死亡 FOV 分离、按住释放、世界 gate 和非零 shake／nausea 的矩阵组合；后两项数学证明不能替代实机来源验证。

2026-11-26 续接中补齐实际游戏线程的模块 actor、原版配置树、持久化、窗口与主题初始化路由，新增 `client_control_key` 精确插入点，使官方 socket 的合成按键进入相同 core 输入路径。新增只读 `/api/v1/client/integration-status` 记录已经发生的相机／lightmap 接入点输入输出、模块 running 与真实 CEF 诊断；不以模块配置值代替已发生的效果。`scripts/Build-LiquidBounce.ps1` 和 `scripts/Start-LiquidBounce-Test.ps1` 提供完整源码构建及独立测试目录。

2026-11-26 的最终五模块实机报告为 `build/game-runs/native-integration-09/native-integration-validation.json`：58 项自动检查全部通过，scope 为 `full_integration`，`fullIntegrationPassed=true`。除前三模块的实际 FOV／gamma／Night Vision 与 Zoom 按键释放外，新增真实非零 walk 和 hurt 的取消／恢复及分支隔离、死亡 roll 取消而死亡 FOV 不变、nausea spin 保留，以及状态恢复。原主题经两次真实 Ctrl+R 文档重载后，官方截图中的 Render 展开并按原版顺序显示五行，独立人工结果记录为同目录 `ui-visual-review.json`。游戏正常退出，进程检查无该次游戏或 CEF 残留。早期三模块 31 项基线保留在 `native-integration-01/`。

整合构建记录为 `build/integrated-build.json`，源码与可执行文件摘要均与最终报告绑定。控制仅使用官方 socket／ctl 与真实本地 REST；人工截图复核不参与自动游戏控制，未使用屏幕识别或坐标点击。真实运行检查了 `bob * shake` 组合，但固定客户端没有官方非零 native shake 生产者，该场景为 `not_exercised`；非零 shake 保留仅有源码与矩阵单元测试证据。View Bobbing=false、保存后重启及全部设置组合没有因本次通过而视作已完整实机覆盖，详细范围见 `render-module-validation.md`。

原版 Java 客户端与 Rust 客户端的相同输入、世界及 tick 差分运行仍为 `not_run`；独立 Java 数值向量与 Rust 实机诊断不等于该项验收。全部 236 模块、完整界面、共享管理器和网络语义尚未完成移植。`.hnmod` 是固定旧版 launcher 使用的源码包；包含 CEF 和 GPU／行为补丁的预览客户端则必须从完整可重建源码和匹配运行资源构建，二者不能混为同一种可直接加载产物。
