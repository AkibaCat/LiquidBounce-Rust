# 原版主题初始化路由核对

固定 LiquidBounce 提交 `ae0eb396c07f22383789b28d5912d714c6ca0a86`；只读原源码，未改主题。

`src-theme/src/App.svelte` 的启动顺序为：静态 `metadata.json` →
`GET /api/v1/client/theme/liquidbounce` → `GET /client/localStorage/all` →
`GET /client/virtualScreen`。WebSocket 重连后会重新同步虚拟屏幕。实际页面
监听的 `virtualScreen` 事件字段为 `screenName` / `action`，不能依据
`events.ts` 中过时的 `type` 字段来发事件。

`TabbedClickGui.svelte` 在挂载时依次调用 `PUT /client/hud-editor`、
`GET /client/info`（读取 OS）、`GET /client/window`（读取 scaleFactor）、
`GET /client/modules/settings?name=ClickGUI` 和 `POST /client/typing`。
`theme/theme_config.ts` 在导入时另行读取 `HUD` 设置的
`SpaceSeperatedNames`；这项缺失会使主题在模块列表前报错。

`ClickGui.svelte` 读取 modules / categories；每个实际模块再读取 settings。
GlobalSettings 会读取 `/client/global`。所有 UI 行为必须仍对应真实的运行状态；
不能为满足页面而生成虚假的玩家、游戏窗口或 233 个未实现模块。

本轮增加的 `liquidbounce-interop::protocol` 处理真正注册模块的 REST 语义，
`ModuleAccess` 由 Horizon 游戏线程实现。模块 POST 忽略请求中的 enabled 字段，
切换实际当前状态；PUT / DELETE 选择启用 / 禁用，重复时返回 403。名称大小写
不敏感，但原 ModuleManager 的查找不使用 aliases。配置 PUT 的原始 JSON 树交给
真实适配器执行，并且事件应来自实际修改，避免 HTTP 层与键绑定重复广播。

`UiState` 独立保存已实现的 UI 展示状态：从真实主题 metadata 读取 ARGB 默认颜色，
在隔离目录持久化 localStorage 和有限 ClickGUI/HUD 展示选项。`set_screen` 必须由
实际屏幕打开/关闭回调调用；typing、hud_editor_selected、page_keeps_escape
供原生输入捕获读取。它不持有或猜测玩家、世界、fps、会话、窗口状态。
ClickGUI 缓存/独立窗口、全部 HUD 组件和全局功能仍需继续移植。

`examples/theme-preview.rs` 是可独立运行的原主题 CEF 传输回归入口；明确输出
`presentation_only`、`registered_modules: 0`，并报告 `inGame: false`。
此预览的 window 数据仅表示命令行配置的预览 viewport，不冒充游戏窗口。
原版 HTML/JS 能在该入口加载只能证明主题传输与浏览器路径，不能替代运行游戏验证。

真实 TCP 测试验证了原版 POST / PUT / DELETE 状态语义、未知模块和别名拒绝、配置树
传递与读取；UI 测试读取 metadata 的真实默认颜色并在重新加载后核对布局持久化，
同时确认过期 screen acknowledgement 不能阻止新的游戏屏幕处理 Escape。
