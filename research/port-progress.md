# 全部 236 模块的移植进度

更新日期：2026-11-26。固定 LiquidBounce `ae0eb396`、Horizon `9f42064`、Horizon-Mixins `5a64af9`，不修改 `upstream/`。

用户目标仍为全部 236 模块及随游戏语言切换的简繁中文。以下均为阶段证据，不能当作任务完成。

| 层级 | 当前证据 |
| --- | --- |
| 原版范围 | `module-parity.json`，236 个真实 builtin 模块，不含模式和未注册对象 |
| 共享源码 | 190 个真实模块已接线并通过官方 native 与 engine 类型检查；最近证据为 `../build/integrated-check-190-final.json`。接线与类型检查不等同于全部功能和实机验收 |
| 最近整合测试 | 76 模块 `../build/test-engine-76-03.log`，104/104 tests通过；固定报告 `../build/integrated-test-engine-76-03.json`，含真实TextField/EditBox渲染消费及ElytraSwap实际退出入口测试 |
| 最近配置快照 | `../build/config-snapshot-73-01/report.json`，passed=true，生产配置 tests 及 Clippy -D warnings 通过，无注册shim |
| 客户端构建 | 76模块成功：`../build/build-game-76-01.log`，退出0；`../build/integrated-build-76-01.json`，EXE SHA256 `86e073992e0e2bfbe587c8f66682cb35541dc8f83da819509a1adf14a617c47d`。含服务端 USE_COOLDOWN 编码修复，run44实机已通过 |
| 最近通过真实游戏 | `../build/game-runs/native-modules-43/`，136 checks通过且退出0；TextFieldProtect、GUICloser、ElytraSwap 定向行为通过，六张截图已实际查看 |
| 最新界面验收 | `../build/game-runs/native-ui-28/`，133 checks通过，进程退出0；四张实际界面截图已查看，见 visual-review.json |
| 最近通过游戏运行 | `../build/game-runs/native-integration-22/`；FreeLook鼠标取消/镜头、SnapTap四种按序、SmartEat实际食用、SkinDerp选项恢复、VehicleBoost下车、BlockBounce实际弹跳均通过 |
| 全部模块实机验收 | 尚未完成；`fullIntegrationPassed=false` |
| 全客户端差分 | Java原客户端/Rust客户端完整差分未运行；独立Kotlin/JAR oracle不等同于此项 |

当前175项接线包含已实现原生行为，也包含尚有下游依赖和共享服务缺口的模块。最新可运行EXE仍为76模块版本；190项尚未生成新的完整EXE。AutoWeapon OnTarget已接Aimbot/CrosshairAttacker，KillAura实际调用者仍开发中。完整原InventoryManager的跨模块priority/FIFO队列、atomic chain、carried-stack恢复、5ms光标协程、拒绝锁及真实全局配置已接入；BetterInventory 已接入共享源和68 EXE，实际高亮/容器/冷却与光标场景待通过。AutoTool动态/静态真实挖掘、Vomit生存库存与创造随机方块丢弃、VehicleOneHit实际船/矿车销毁在run32已通过。Chams有GPU像素用例与JAR dispatch oracle；run32可见目标在Normal/Image保留原材质，run34新增真实石墙对照，禁用完全遮挡、Normal原材质穿墙、Image纹理穿墙四张图已实际查看并记录 chams-visual-review.json；完整视觉等价仍为false。

新增独立证据：

- SmartEat：`../build/smart-eat-tests/result.json`，47 tests及Clippy；真实1658 item /27 use-owner dispatch、PASS-only事件、饥饿/食品选择与全局FIFO stable callback接入。
- SkinDerp：`../build/skin-derp-source-01/report.json`，原Kotlin源码、真实coroutines/JDK25 RNG，844 command rows；实际7个modelPart选项与启用快照恢复。
- EntityControl：`../build/entity-control-04/report.json`，6 tests及Clippy、320 control/40 Float getter oracle；实际saddle getter及LocalPlayer jump-scale隔离。
- SnapTap：`../build/snap-tap-tests/result.json`，17 tests及Clippy；实际KeyBinds/winit和生成Host bridge，原始键盘事件先于模块绑定切换，无禁用/世界重置。
- HandDerp：`../build/hand-derp-packets-02/report.json`，真实Session/Pipe解码及普通accepted packet观察，不能解释为服务端ACK。
- 统一CoroutineTicker FIFO现提前至native tick HEAD，在SilentHotbar计时器和CombatWindow递减之前，七模块共享；whole-tick嵌套深度保护保留。
- BlockWalk：`../build/block-walk-08/report.json`及`../build/block-walk-combined-07/report.json`；真实1286 block registry，AirJump GhostBlock组合2880 cases通过。
- GhostHand：`../build/ghost-hand-tests/result.json`，9 tests及Clippy，未改原TweakedMethods.java/MC shapes 4608 cases通过。
- ExtendedFirework：`../build/extended-firework-05/report.json`，58 tests及Clippy，未改原Kotlin/Vec3dAdapter、实际MC视角和原Java公式，442 imports与576 boosts raw bits匹配；完整transformed rocket consumer/generated dispatch通过。
- Strafe：`../build/strafe-02/report.json`，6 tests及Clippy，未改原Kotlin/真实MC 69,125 rawbits cases通过；主配置快照已修File/Interop测试预期。
- ElytraRecast：`../build/elytra-recast-tests/result.json`，12 tests及Clippy；未改原Kotlin与Java mixin，362实际MC stack变体、8192 tail及2048 direct cases；夹具remote_scope独立模块验证完整限定路径。
- FastBreak：`../build/fast-break-09/report.json`，7 tests及Clippy，未改原Kotlin、28,800 cases、实际1658物品及普通nested packets；Remote字段真实sibling隐私边界编译通过。
- InventoryManager：`../build/inventory-manager-source-05/report.json`，479条原Kotlin commands；`../build/inventory-manager-13/report.json`，14 tests及Clippy；actor 4 tests及Clippy、完整真实API engine check见 inventory-freeze.json。
- Vomit：`../build/vomit-source-03/report.json`，2580原Kotlin vectors；`../build/vomit-registry-02/report.json`，1286 blocks、78 air items、1658 items。
- VehicleOneHit：`../build/vehicle-one-hit-07/report.json`，3 tests及Clippy，161实际EntityTypes、15,360原Kotlin cases；完整变换后的本地boat/cart入口覆盖取消和目标集合改变。默认单人经独立服务端线程与真实内存连接处理普通攻击包；run32确认禁用时船/矿车存活，启用时实际销毁。
- run23/24均为62模块EXE，进程退出0但verifier未通过，不能算整轮成功。run23 AntiAFK夹具原先误以为yaw固定180，已按真实不wrap多周期逻辑修正。run24 BlockWalk真实雪层碰撞及GhostHand石后箱子通过，Strafe实际停位通过，但ground摩擦在外部采样前把存储速度归零；夹具已换高空，由run25重新验证。

中文覆盖：236模块名称、3905字面声明/2439 distinct captions，简繁缺失0；实际`lang::code()/generation()`驱动界面切换。繁体允许`zh_tw → zh_cn → en_us`回退，未声称人工全面审阅。源码现有279条owned设置帮助，包含Timer/TerrainSpeed、HighJump和ItemScroller；开发中模块的帮助仍持续补齐；`../build/theme-localization-bootstrap-02.log`验证通过。66 EXE具体烘焙的帮助条目数须按其运行快照确认，不能用当前资源文件推断。

`../build/theme-build-bootstrap-01.log`显示派生中文主题构建及界面类型检查通过；已修App在persistent storage完成前挂载Router导致空布局覆盖的竞态。最新主题output SHA-256 `d5b68ed519aec484765cf341a6379bd63de2877ead02b8973aa33c6b00712859`。run28用66 EXE完成两次Ctrl+R与简繁英切换，original-clickgui、zh_cn、zh_tw、en_us四张截图均已实际查看，文字可读、Render展开与其余分类无重叠；视口外行和全部设置未视觉验收。

run25因SmartEat夹具hunger255导致死亡，已改hunger63并在food<18立即清除；run26真实SmartEat/BlockWalk/GhostHand/高空Strafe通过，随后摔死污染SleepWalker，已加高空creative与落地survival隔离。run27因上述主题存储竞态失败，run28界面验证已证实修复。run29 AutoWeapon四个客户端tick内尚未收到目标健康metadata，run30改等待真实目标生命值下降后已通过；run30第二次AutoTool挖掘遇到异步setblock尚未同步，run31已改为等待实际stone状态再执行原挖掘断言。失败记录均保留。

失败记录全部保留：run15页面首次加载失败、run18 controller wait过长、run19/20睡眠状态污染相机夹具、run21旧物品恢复遗漏组件。run14/17简繁英截图确已查看，不能替代新58模块客户端与新主题的实机复核。

官方中文游戏资源按固定26.3 manifest/asset index逐项验证SHA-1与尺寸，证据为`../build/resource-preparation-import-evidence.json`及`../build/chinese-resource-validation.json`。完整资源离线检查仍报告既有builtin packs/全景/声音缓存缺口。

正在继续：73 模块累计测试和 fresh 实机行为验证；TerrainSpeed、Timer及完整Timer服务、BetterInventory、AutoBreak、HighJump、ItemScroller 已接入，全部模式实机验收仍待完成。ExtendedFirework/ElytraRecast仍需补专门实机场景；FastBreak定向场景已添加，待通过。剩余模块持续按原源码和真实host逐项实现；未接入项保持not_implemented，不批量注册占位。

run31在创造Vomit用例遇到自然僵尸击杀玩家、重生改变位置，判为失败并保留。run32在召唤测试目标前关闭spawn_mobs并清除自然实体，237 checks通过且退出0。Timer当前已接入共享源码（67项），生产配置快照config-snapshot-67-01通过；test-engine-67-01的85 tests通过。新增世界生命周期接点/诊断后仍需再构建和验收，67 EXE尚未构建和实机验收。

新增68模块实机 run33、run34均判失败并保留。run33在Chams禁用时错误等待不会dispatch的回调，已改检查模块状态与真实tick。run34已完成Chams真实石墙对照，随后眩晕镜头场景160 tick等待超过controller 10秒预算（并行编译负载），未到Timer/BetterInventory；长tick wait预算现为45秒。run34退出0，不能算整轮通过。

TerrainSpeed 第69项已接共享行为/config/actor/生成hook，FastClimb作为原别名。独立 native06 passed（24 tests、Clippy及完整library check），7040 Kotlin原操作、305真实调度与144 push oracle；完整原/新客户端差分未运行。当时HighJump、AutoBreak尚未注册；现已完成共享接线，AutoWalk仍由代理完整开发且未注册。


最新累计记录：69模块TestEngine和EXE均通过。70模块TestEngine首次失败，原因是AutoBreak事件原始值字段误作引用及多余lifetime，已修复；71模块累计90 tests通过，config71通过。72项新接ItemScroller完整设置/actor/native query/reset，config72通过，累计TestEngine进行中。HighJump保留原Mode tag，并新增真实space跳跃、Vulcan延迟清零与偶数tick滑翔实机场景，尚未执行。

69模块run35（native-modules-35）仍失败：Timer Classic、Pulse前两阶段确实执行，第二次常速阶段外部采样超时，未到BetterInventory/TerrainSpeed。失败时实际frame delta约7.18，受并行编译低帧率影响，短阶段可能在同frame内完成。实机夹具已将Pulse阶段延长为60/60 ticks以便跨帧观察，原短阶段Kotlin协程对照保持。修正尚待实际重跑，不能按猜测标通过。

ItemScroller独立source01、config02、consumption02、native02均passed，覆盖原QuickMove唯一模式、每次eligible query随机延迟、wall clock、物理Shift/左键、全鼠标按钮取消、五处render与创造垃圾槽46包。原生累计/实机仍为独立待验步骤。HighJump独立8tests与48条原协程轨迹通过，native02完整library check通过，100-tick waits与should_glide跨生命周期语义保留；完整客户端差分仍未运行。


72累计94tests及EXE构建已通过，EXE SHA256 `f4fa5a07d2d75d09499948ce40ddcec17e885163a60f36d124bf8e5b11d90f41`。run36的AutoBreak、HighJump（Vanilla/Vulcan延迟和滑翔）、ItemScroller真实物品转移场景通过；整轮因Timer创造背包夹具预期错误失败。run37修正生存背包后通过该guard，随后实体位置字段误读而失败；其process monitor先于launch-result启动失败，不能补称进程验收通过。run38零延迟InventoryPlan在绘帧前清lastClickedSlot，观察高亮超时。run39使用原导入延迟，Border/Texture、实际visibleCursor与关闭开关、稀疏shulker预览及过滤/缩放通过；三张截图已实际view_image，局部视觉记录 `better-inventory-visual-review.json`，冷却文字仍pending。全部失败日志保留。

run40 Timer完整定向场景通过：Classic、Pulse60/60完整周期和取消、OnMove、Boost背包/运动/真实攻击、禁用恢复、真实维度替换重置。独立服务端SetTime采样分别20.0/19.92TPS，客户端0.5/3.0实测10.08/30.98TPS；完整记录见 `../build/game-runs/native-modules-40/native-integration-validation.json`。随后FastBreak因原世界草块挡住语义look-at失败，已为夹具清空走廊，整轮仍false。run38/39/40进程监视均确认exit0。

AntiHunger第73项已接persistent Arc服务、priority0包监听（排在MoreCarry/NoPitchLimit前，匹配原注册序）、Sprint priority500之后的INPUT/NETWORK、Mutex配置及正常WebSocket通知。通知语言按发出时捕获并使用原简繁英文案。独立8tests/2280Kotlin步轨迹及native03完整library check通过，config-snapshot73 passed，累计TestEngine进行中；尚未运行73 EXE。GUICloser完整JDK正则、SafeWalk真实模拟和AutoWalk导航仍开发中，均不注册、不计完成。

上述历史条目的当前补充：73累计TestEngine已完成94 tests且通过；73-01与73-02客户端构建均退出0。run41 AntiHunger真实INPUT/NETWORK冲刺拦截及子开关/禁用恢复、石块普通挖掘包、实际游戏语言zh_cn/zh_tw/en_us对应WebSocket通知原文均通过。BetterInventory边框/纹理、可见光标开关、稀疏容器预览过滤与缩放通过，真实雪球使用16→15后冷却显示仍失败；整轮false，FastBreak/TerrainSpeed未执行，monitor确认exit0。run41三张界面截图尚未查看，不能记录该轮视觉通过。

冷却问题已定位到网络item_sync跳过USE_COOLDOWN，客户端丢失自定义组、以minecraft:snowball查找服务端liquidbounce:integration组。owned cooldown-components-patch.py已添加实际float/optional Identifier解码及显式移除marker，73-02成功编译。实际MC26.3 codec与真实Horizon item_sync探针仍在验证：cooldown-components-01取消；02因独立Cargo未继承生产wgpu patches编译失败；03已补齐生产patches/profile并在运行。失败目录保留，尚未宣称修复实机通过。

ElytraSwap独立源码策略新增：`../build/elytra-swap-source-04/report.json`通过，852场景对照未改原Kotlin及真实coroutines挂起/取消，Rust tests和Clippy通过。原hotbar→inventory→offhand顺序、启用时捕获方向、取消后保留方向、空胸槽两次pickup及perform完成后disable均保留。实际共享InventoryManager接线、原生tag/component提取与中文配置尚待完成，未注册、不计入73项。新ExtendedFirework与ElytraRecast专门实机场景已写但尚未执行。

冷却探针03已完成且通过：`../build/cooldown-components-03/report.json` 对照实际MC26.3 UseCooldown.STREAM_CODEC的40行bytes/ticks，真实Horizon客户端item_sync、Template wire、冷却组/map及显式移除与prototype fallback。它只覆盖客户端方向。随后run42共100 checks仍整体false：实际雪球16→15通过，但客户端冷却progress=0/end=null/tick=null；FastBreak/TerrainSpeed/ExtendedFirework/ElytraRecast均未到达，进程monitor确认exit0。继续定位发现真实 `lan::guest::stack_to_network` 的built-in编码列表缺少use_cooldown，命令物品在服务端发出时丢组件。完整server→wire→client探针04现已通过40行真实MC编码逐字比较与移除/prototype测试（`../build/cooldown-components-04/report.json`）；修复尚待新EXE实机验证。run42三张边框/纹理/稀疏容器图已实际view_image，局部记录见`better-inventory-visual-review.json`，不含冷却或整轮通过声明。

ElytraSwap共享接线和中文设置帮助现已完成：真实InventoryManager完成后才同步disable、取消保留方向、完整PlayerInventoryConstraints及实际glider/chest_armor组件标签。config01及actor02通过；74累计TestEngine为98 passed/1 failed，失败是退出用例只调用world更新而未调用实际disconnect入口。canonical测试已改用真实disconnect_sync，保留原断言，等待下一次整合验证；失败报告保留`../build/integrated-test-engine-74-01.json`。主题74构建及类型检查通过，236模块中文名称仍完整。

TextFieldProtect正在开发，尚未注册：`../build/text-field-protect-source-01/report.json`通过6122条未改原模块/真实JDK25 Kotlin Regex对照，Char.repeat逐字抽取原函数体。包含原displayPos计数、分段匹配、负索引和有序nullable规则；实际render hook、原Gson linked-set配置与游戏截图验证仍待完成。导航walking support05（12298条）和Flying03（927条）已冻结，完整AutoWalk仍未注册。

上述TextFieldProtect状态的当前补充：共享配置、registry/actor和真实RenderHooks已接为第76项，尚未整合编译和实机验收。`../build/text-field-protect-config-source-01/report.json`已通过原Gson/ListValue/Serializer的23输入组、完整canonical配置快照82tests及Clippy，无注册shim。原语义是每个applyFormat子串独立匹配，mask长度取displayPos的UTF-16计数，未改输入值，multiline不受原EditBox mixin影响。新增实际TextField/EditBox/append_field消费测试和聊天框实机场景，待执行。生成器多字段事件的rustfmt尾逗号误报已在`../build/text-field-render-generation-02/report.json`精确复现并验证修复，保留真正签名变化的失败检测，Clippy通过。01夹具因未复制generator子模块失败并保留。

中文76资源检查`../build/theme-localization-76-01.log`通过：236名称、2292简体label、1112繁体label、293 owned设置帮助；`../build/theme-build-76-01.log`主题构建和类型检查退出0，output SHA256仍为`d5b68ed519aec484765cf341a6379bd63de2877ead02b8973aa33c6b00712859`。这些是资源覆盖证据，不替代新客户端语言切换和截图验收。

76累计首次测试已真实运行并保留失败：`../build/integrated-test-engine-76-01.json`。生产构建此前没有调用fill_hooks，导致lower render生成声明没有TextFieldFormatEvent；`Build-LiquidBounce.ps1`已增加官方`cargo +stable run -p xtask -- mod-api`，并记录生成文件/构建driver的SHA256。76-02已走过真实生成步骤，随后因GUICloser的client_container_menu_text跨sibling私有访问E0624失败（`../build/integrated-test-engine-76-02.json`），正在修owned可见性；尚未宣称76客户端构建或实机通过。


2026-11-26 加速开发策略（用户最新要求）：全部236项持续实现；每个新增模块只进行一次有针对性的编译/短smoke，不累计重跑全部模块、不扩大oracle矩阵；功能开发完成后再统一全量测试。

76模块整合已收尾：TestEngine76-03为104/104 tests通过；EXE76-01构建成功。run43共136 checks通过，真实TextField/EditBox输入未改而显示掩码、GUICloser英/简/繁真实标题及规则、ElytraSwap真实hotbar/main/offhand库存动作和自动关闭均通过。旧窗口到新取消窗口的上下文只有真实source消费夹具，未实机。

run44共112 checks整体false：BetterInventory冷却组件服务器→wire→client修复经真实雪球16→15确认，25s冷却文字、边框/纹理/光标/稀疏容器预览通过，4图已实际查看；FastBreak空手OnlyTool guard及damage阈值真实挖掘通过。TerrainSpeed梯子夹具的两种speed样本均静止onGround，原因未定位，Ice/Water未到达，不能标通过。后续仅定向处理该场景。

run45整体false且退出0：真实胸甲elytra滑翔、服务器火箭使用、禁用时正常consumer已通过，Normal multiplier实际为[0.1,1.65,0.5]。失败由夹具要求ExtendedFirework的Normal tag导致；已核对原模块没有tagBy，夹具改为tag=None。Custom、禁用速度恢复及ElytraRecast尚未执行，保留失败记录，待最终统一运行。

新增共享接线83项（尚待native83编译结果）：ResetVL、NameCollector、Kick、AutoLeave、TargetLock、YggdrasilSignatureFix、Damage。AutoLeave仅1个短测试通过（其余已有tests均过滤）；TargetLock冻结source01为8场景72步、tests01为10 tests；Yggdrasil实际patched skins::unpack单次smoke通过，验证签名状态/域名检查/127.0.0.1瘦手臂/禁用恢复与坏payload；Damage三模式单次smoke通过。NameCollector与Kick通过真实Host接点执行，本批无需给每个低影响动作增加镜像测试。

native80首次Check失败保留在`../build/check-native-80-01.log`及`../build/integrated-check-80-01.json`：ResetVL owned接点误用glam::DVec3；native83随后将其误改为glam::Vec3，仍失败。实际Horizon速度类型为horizon_core::geometry::Vec3，三个分量均为f64。canonical现已使用该真实类型，Damage的y赋值也恢复f64。直接engine check因未加载官方VS/cmake环境失败（`../build/check-engine-83-01.log`），进程均已收尾；下次使用官方Build脚本，不运行累计TestEngine。

当前共享接线88项：上述83项加AutoDisable、Plugins、PotionSpoof、NoClip、TrueSight。每项只做短smoke，均已有独立通过证据；简繁中文资源已合并。AutoDisable及NoClip实际位置回退包括已取消包，网络线程只入队，Game actor执行同步disable与本地清理；完整native生成和编译尚待本批Check，未将88项记为客户端通过。

2026-11-26：已接线103项并通过官方native Check（build/integrated-check-103-01.json，exitCode=0）。新增范围包括ResetVL至Teams以及Parkour、SafeWalk；module-parity.json已同步103项实现状态，保留既有实机/差分状态，仅新增部分记录focused_checks_game_pending。该编译结果不代表103项实机通过。

Spammer与Breadcrumbs已接线为105项。Spammer文件/格式/UTF-16短smoke一次通过；Breadcrumbs生产runtime/config/GPU独立短测通过，九种shader实际读回非零像素。官方105-01整合首次因gfx局部Frustum误传给Mat4失败；owned已修为views.primary().view，未重跑GPU短测。PingSpoof队列边界/禁用释放/旧连接丢弃短smoke一次通过，真实接线已完成；下一批官方Check待结果。用户要求的最终全量测试尚未开始。

2026-11-26：共享注册已推进至129项，官方 native＋engine Check129-01通过（build/integrated-check-129-01.json，exitCode=0）。新增114→129包括NewChunks、LiquidWalk、Sneak、Step、Tracers、ChestCleaner、TriggerBot、AutoClicker、ProphuntESP、VoidESP、AntiCheatDetect、NoWeb、ProtectionZones、Eagle、Criticals；ledger与verifier已同步。各模块仅做过一次定向短测，未重跑累计全量。125/126生成失败与128编译失败记录保留，129成功不等同全实机或差分通过；最新完整EXE仍是76项。剩余107项持续开发。

2026-11-26：官方 native＋engine Check139-03已通过（build/integrated-check-139-03.json），ledger/verifier同步139项。130→139为ReportHelper、NoSlow、VehicleControl、AutoDeposit、FastExp、AutoQueue、Replenish、Hats、Wings等本批真实实现；历史139-01/02失败保留。Crosshair、InventoryCleaner、Blink随后接线至142项；142-01/02因InventoryCleaner CollisionEntity/CollisionContext/CollisionItem未限定路径而失败，owned现已全部修正，等待下一批串行Check。142尚未记为编译通过；最新完整EXE仍为76项，最终全量测试尚未开始。

2026-11-26：共享152项已通过官方 native＋engine Check152-01（build/integrated-check-152-01.json，exitCode=0）。新增140→152包括Crosshair、InventoryCleaner、Blink、AutoDodge、Macros、ChestStealer、HoleESP、AirPlace、MurderMystery、AutoArmor、Particles、FlagCheck、AutoWindCharge，简繁显示与帮助资源已接入。149-01/02及150-01失败均保留，150 native通过后engine命令Host可变借用已修。本批只做新增项定向短验和整合编译，未运行累计全量；最新EXE仍为76项，236开发完成后再统一全量和实机。

2026-11-26：官方 native＋engine Check155-02通过（build/integrated-check-155-02.json，exitCode=0），ledger/verifier已同步155项。新增Offhand、LogoffSpot、PacketLogger；155-01的Offhand配置字符串借用失败已修，报告保留。PacketLogger读取真实typed字段和最终取消状态，未知payload明确保留原bytes；不冒充JVM反射结果。AutoBuff和AntiStaff随后接入至157项，Check157-01正在进行。继续仅做新增模块一次短验，最终全量尚未开始；最新完整EXE仍为76项。

2026-11-26：官方 native＋engine Check160-02通过（build/integrated-check-160-02.json，exitCode=0），ledger/verifier已同步160项。新增AutoBuff、AntiStaff、StorageESP、Notifier、ProjectilePuncher；157-01的AutoBuff Arc/向量、157-02的hand整数、159-01的StorageESP接口生成、160-01的StorageESP/Notifier原生类型错误均已定向修复，历史日志保留。159-01在生成阶段失败且未生成integrated报告，没有挪用旧报告冒充该轮。AntiStaff补丁同时保留BetterTab仅显示名字时不追加game mode的行为。新增功能只做一次定向短验；未运行累计全量，最新完整EXE仍为76项。Notifier真实AntiBot过滤依赖正在实现，不能将当前消费者集合算作AntiBot。XRay、AutoBow、共享TargetRenderer、AntiBot继续开发。

2026-11-26：官方 native＋engine Check180-04 通过（build/integrated-check-180-04.json，exitCode=0）；ledger/verifier 已同步180项。175→180为TotemEffect、StrongholdFinder、KillAura、SwordBlock、PotionFX。新增及变更项定向证据已只读汇总到 build/batch-180-smoke-01/report.json；KillAura配置首次3过1失败，仅该失败项修复后复测成功，未重跑通过项。另补JumpEffect真实场景深度GPU单项通过。随后InventoryTracker、TNTTimer、Radar、NameProtect、AutoCrafter接线至185项，待本批统一官方编译与短测；尚未标为编译通过。最新完整EXE仍为76项，最终统一全量及实机尚未开始。


### 2026-11-26 批量检查 185 项

- InventoryTracker、TNTTimer、Radar、NameProtect、AutoCrafter 联合 native/engine check 通过：`build/integrated-check-185-07.json`（exitCode 0）。
- 本批 42 条 focused 通过，证据只读汇总于 `build/batch-185-smoke-01/report.json`，未重跑 180 项旧批。InventoryTracker 配置首轮 1 过 1 失败，修正测试对 Gson boolean 字符串语义的误解后只重测该项；TNTTimer 首次夹具带入无关 HUD GPU tests 未能编译，排除无关tests后 5 过 1 失败，生产 WGSL 比较式补括号后只复测该失败项通过。WGSL 更新纳入下一批官方编译。
- NameProtect 已和原 Kotlin oracle 的 529 向量比较通过；未重跑参考程序。
- ledger/verifier 同步到185。仍没有新链接 EXE/全量游戏通过声明：最新完整 EXE 仍为76项版本（SHA256 `86e073992e0e2bfbe587c8f66682cb35541dc8f83da819509a1adf14a617c47d`）。
- 下一批保持5～8项一起编译；批内只静态检查和prepare，各项短测只在本批编译后运行，全部236开发后统一全量测试。


### 2026-11-26 批量检查 190 项

- CustomAmbience、ItemTags、ESP、ItemESP、BedPlates 联合 native/engine check 通过：`../build/integrated-check-190-final.json`（exitCode 0）。
- 本批 focused 43 项通过，证据汇总于 `../build/batch-190-smoke-01/report.json`；CustomAmbience 8、ItemTags 6、ESP 14、ItemESP 8、BedPlates 6、BedPlates choice labels 0、共享 OutlineGlow GPU 1。
- ItemESP 与 BedPlates 首次 fixture 暴露测试夹具缺少共享解析函数；仅修正夹具并重新运行失败组，之前已通过的组未重跑。
- 官方成功检查仍只表示 190 项 native/engine 类型检查通过；完整 EXE 仍是 76 项版本，236 项开发完成后再统一全量测试。
