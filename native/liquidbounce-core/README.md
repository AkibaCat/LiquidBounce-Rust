# LiquidBounce Rust core

GPL-3.0-or-later 基础库，依据固定上游 LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86` 的实际实现移植。仅使用 Rust 标准库，独立构建和测试，不依赖 Horizon 或 JVM。

## 已实现的基础行为

- 以 Rust payload 类型分派可变事件；优先级降序、相同优先级保持注册顺序，调用时取注册表快照。
- 每个处理器调用前重新检查 running gate。事件取消后继续调用剩余处理器；完成后禁止再取消。普通 Rust unwind panic 计数后继续，`panic=abort` 不适用。
- 明确区分未知事件、已注册但无处理器的事件，以及 self-destruct/error-screen 抑制状态。
- 模块 enabled/running 区分、world 和 player 同时存在的真实世界 gate、第一次进入世界时的延迟生命周期、disableOnQuit、notActivatable/disableActivation。
- TOGGLE、HOLD、SMART 绑定；键盘 SMART 以 repeat 区分长按，鼠标 SMART 以 200ms 为边界。GUI 内仍处理释放；键盘按下受 debug modifier 抑制，鼠标与原版相同不受这个抑制。
- 逻辑修饰键组合匹配、最后一侧修饰键释放、Unbound 和外部绑定规则。

模块注册表初始为空。本库没有注册任何战斗、移动、渲染或网络功能；名字和状态只应由已经完成真实实现的模块注册。Horizon 桥接必须提供实际 world/player 状态、按键映射、已按下的左右修饰键和时间。

## 实现边界

`ModuleEffect` 仍是基础效果 API，状态在返回前提交。真实模块桥接应采用下面的同步 API，不能用延后执行效果代替生命周期顺序。嵌套 ValueGroup/Mode、配置和通知、事件流、Kotlin 协程及完整 236 个模块的实际实现仍由桥接逐项接入；同步回调提供它们的正确执行位置，没有生成虚假实现。

## 同步生命周期与重入

`set_enabled_sync`、`handle_keyboard_sync`、`handle_mouse_sync`、`update_world_sync` 和 `disconnect_sync` 接受 `&impl ModuleCallbacks`。`ModuleCallbacks::invoke(&self, &mut ModuleRegistry, ModuleCallback) -> Result<(), String>` 接收实际注册表的可变引用，内部不持锁，也不保留模块借用。回调可以通过传入的注册表和同一个 executor 再次调用同步 API。executor 使用共享 `&self`，自身可变行为状态可按需要置于 `Cell`/`RefCell`；跨回调保留内部可变借用会阻止该 executor 自己的重入。

运行于游戏内时，执行顺序为：

1. 先设置 `calledSinceStartup`，但保留原 `enabled` 字段。
2. 禁用时执行 `CancelSequences`；随后执行 `OnEnabled` 或 `OnDisabled`。
3. `UpdateChildren` → `Activation`；`disableActivation` 为真则接受状态变为 false。
4. 正常激活时，如果实际 `loading` 为假则执行 `Notification`，随后执行 `Toggle`。
5. 提交接受的 `enabled`。
6. `ValueChanged` → 接受值为 true 时的 `EnabledEffect` → `PublishEnabled`。

`UpdateChildren` 是原版嵌套组递归所在的位置；`PublishEnabled` 是原版 `stateFlow.value = currT` 的位置。桥接必须在这些回调内执行相应的真实工作。进入世界的延迟启动在临时 loading=true 下直接执行生命周期，不重新提交返回值，不产生 ValueChanged/EnabledEffect/PublishEnabled；这也保留了 passive 模块的原版启动边界。loading 状态通过 `set_loading` 输入，启动结束恢复之前的值。

与 `Value.set` 一样，相等检查只在调用入口执行。重入可能先提交另一次操作，外层操作随后仍提交自己的接受值；已有重入写入和回调副作用不会回滚。后提交的 ValueChanged 回调若再次修改状态，外层 EnabledEffect/PublishEnabled 仍接收原先接受的值。按键批处理中，上一模块的回调完成后才读取下一模块当前的绑定和 enabled，避免对过期快照执行切换。

原版 `ToggleableValueGroup` 吞掉协程取消异常并继续禁用；同步 API 在 `ModuleSyncReport.failures` 中记录它。其他提交前回调失败按 `Value.runCatching` 拒绝外层提交，仍返回报告，保留 `calledSinceStartup` 及已经发生的重入写入。提交后的失败对应原版位于 runCatching 外的 `onSuccess`，返回 `ModuleSyncError::AfterCommit`，保留提交；输入处理停止，disconnect 则逐模块捕获后继续。普通 unwind panic 映射为相同阶段的结构化错误；`panic=abort` 无法捕获。

桥接应在回调内使用传入的注册表，避免重新锁住包含它的外层 State Mutex。若 Host 调用可能重新进入整个桥接，也必须安排桥接自身的状态所有权，不能在持外层非重入锁时调用 Host；本库的同步 API 不会替桥接管理该锁。

当前注册表的 unregister 移除并释放其模块状态；如果回调注销正在提交的模块，后续提交返回 UnknownModule。Kotlin 的外部对象引用在注销后仍存活，这种已注销对象的独立 Value 生命周期未在本注册表中表示。任意附加 Value listeners 的转换、immutable 值以及 coroutine/StateFlow 实现同样仍需配置系统接入。

名称只接受已经核对的内建 ASCII 命名，按 ASCII 大小写无关顺序排列；Java 对任意 Unicode add-on 名称的排序尚未移植。`InputKey::Keyboard` 接受原版 SDL scan code，Horizon 使用 winit，映射由桥接负责。`Modifiers` 使用本库的逻辑位，不接受未转换的平台原始 bits。

本库中的 world gate 仅由调用方观察值更新，既不创建世界也不把本地状态变化当作服务器确认。

## 原版依据

| 行为 | 固定上游文件 |
| --- | --- |
| 事件完成／取消 | `event/Event.kt` |
| 优先级、快照、稳定顺序 | `event/EventHookRegistry.kt` |
| 分派 gate、异常后继续 | `event/EventManager.kt` |
| enabled/running、延迟生命周期 | `features/module/ClientModule.kt` |
| 绑定、200ms 鼠标边界、断线／首次世界处理 | `features/module/ModuleManager.kt` |
| 修饰键／Unbound | `utils/input/InputBind.kt` |
| 世界 gate | `utils/client/ClientUtils.kt` |
| 生命周期、同值不重复改变 | `features/misc/Toggleable.kt`、`config/types/Value.kt` |

这些文件的完整路径位于本项目 `upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/`。

## 验证

```powershell
cargo test --manifest-path native/liquidbounce-core/Cargo.toml --offline
```

测试用独立预期值验证事件顺序／取消／快照、gate、生命周期、GUI 与 debug modifier、SMART 短按／长按、200ms 边界和修饰键释放。不构建 Horizon 游戏，也不声称已验证完整游戏行为等价。
