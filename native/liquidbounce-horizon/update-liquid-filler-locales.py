#!/usr/bin/env python3
"""Populate LiquidFiller and its complete non-support Placer in both Chinese locales."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.liquidFiller'
LABELS={
 'LiquidFiller':('填充液体','填充液體'),'PlaceIn':('填充液体种类','填充液體種類'),'Water':('水','水'),'Lava':('熔岩','熔岩'),
 'PlaceOrder':('填充顺序','填充順序'),'Random':('随机顺序','隨機順序'),'CloserFirst':('从近到远','由近至遠'),'FurtherFirst':('从远到近','由遠至近'),'BottomTop':('从下到上','由下至上'),'TopBottom':('从上到下','由上至下'),
 'UseSponge':('使用海绵','使用海綿'),'Filter':('方块筛选方式','方塊篩選方式'),'Whitelist':('白名单','允許清單'),'Blacklist':('黑名单','封鎖清單'),'Blocks':('填充方块','填充方塊'),
 'Placer':('方块放置器','方塊放置器'),'Range':('交互范围','互動範圍'),'WallRange':('穿墙范围','穿牆範圍'),'Cooldown':('放置冷却','放置冷卻'),'Swing':('挥手显示','揮手顯示'),'SwingMode':('挥手显示','揮手顯示'),
 'ConstructFailResult':('构造备用点击结果','建立備用點擊結果'),'Sneak':('潜行时长','潛行時長'),'Ignore':('忽略限制','忽略限制'),'OpenInventory':('打开容器界面','開啟容器介面'),'UsingItem':('正在使用物品','正在使用物品'),'SlotResetDelay':('槽位恢复延迟','欄位復原延遲'),
 'RotationMode':('转向模式','轉向模式'),'Normal':('正常转向','正常轉向'),'None':('不转向','不轉向'),'PostMove':('移动发包后执行','移動發包後執行'),'SendRotationPacket':('发送转向数据包','傳送轉向封包'),'Placements':('每次操作放置数','每次操作放置數'),
 'DestroyCrystals':('销毁阻挡水晶','銷毀阻擋水晶'),'Enabled':('启用','啟用'),'Delay':('攻击间隔','攻擊間隔'),'Instant':('立即执行','立即執行'),'IgnoreOpenInventory':('忽略容器界面','忽略容器介面'),
 'DoNotHide':('显示挥手','顯示揮手'),'HideForBoth':('双方隐藏挥手','雙方隱藏揮手'),'HideForClient':('本地隐藏挥手','本機隱藏揮手'),'HideForServer':('服务器隐藏挥手','伺服器隱藏揮手'),
 'TargetRendering':('待放置方块渲染','待放置方塊繪製'),'PlacedRendering':('已放置方块渲染','已放置方塊繪製'),
 'Clump':('合并相邻方块','合併相鄰方塊'),'StartSize':('初始缩放','初始縮放'),'StartCurve':('进入缩放曲线','進入縮放曲線'),'EndSize':('结束缩放','結束縮放'),'EndCurve':('退出缩放曲线','退出縮放曲線'),'FadeInCurve':('淡入曲线','淡入曲線'),'FadeOutCurve':('淡出曲线','淡出曲線'),'InTime':('进入时长','進入時長'),'OutTime':('退出时长','退出時長'),'Color':('填充颜色','填滿色彩'),'OutlineColor':('轮廓颜色','輪廓色彩'),
 'Rotations':('转向设置','轉向設定'),'AngleSmooth':('角度平滑','角度平滑'),'Linear':('线性','線性'),'Sigmoid':('S形曲线','S形曲線'),'Acceleration':('加速度','加速度'),'HorizontalTurnSpeed':('水平转向速度','水平轉向速度'),'VerticalTurnSpeed':('垂直转向速度','垂直轉向速度'),'Steepness':('曲线陡峭度','曲線陡峭度'),'Midpoint':('曲线中点','曲線中點'),
 'YawAcceleration':('水平转向加速度','水平轉向加速度'),'PitchAcceleration':('垂直转向加速度','垂直轉向加速度'),'DynamicAccel':('动态加速度','動態加速度'),'CoefDistance':('距离系数','距離係數'),'YawCrosshairAccel':('准星水平加速度','準星水平加速度'),'PitchCrosshairAccel':('准星垂直加速度','準星垂直加速度'),'AccelerationError':('加速度误差','加速度誤差'),'YawAccelError':('水平加速度误差','水平加速度誤差'),'PitchAccelError':('垂直加速度误差','垂直加速度誤差'),'ConstantError':('固定角度误差','固定角度誤差'),'YawConstantError':('水平固定误差','水平固定誤差'),'PitchConstantError':('垂直固定误差','垂直固定誤差'),'SigmoidDeceleration':('S形减速','S形減速'),
 'MovementCorrection':('移动修正','移動修正'),'Off':('关闭','關閉'),'Strict':('严格修正','嚴格修正'),'Silent':('静默修正','靜默修正'),'ChangeLook':('改变视角','改變視角'),'ResetThreshold':('复位角度阈值','復位角度門檻'),'TicksUntilReset':('复位等待刻数','復位等待刻數'),
}
# English, Simplified Chinese, Traditional Chinese; shared captions retain exact source identifiers.
DESCRIPTIONS={
 'PlaceIn':('Select the source fluids to fill. At least one type stays selected.','选择要填充的源液体，至少保留一种。','選擇要填充的源液體，至少保留一種。'),
 'PlaceOrder':('Sort newly found source positions before appending them to the retained placement queue.','对新发现的液体源排序，再追加到保留原顺序的放置队列。','排序新發現的液體源，再附加至保留原順序的放置佇列。'),
 'UseSponge':('For water sources, search for a reachable replacement position from which a dry sponge can absorb the source.','对水源寻找处于范围内、可放置且海绵能吸收到目标水源的位置。','為水源尋找位於範圍內、可放置且海綿能吸收目標水源的位置。'),
 'Filter':('Choose whether Blocks lists allowed or excluded normal filler blocks.','选择方块列表表示允许使用还是排除使用的普通填充方块。','選擇方塊清單代表允許或排除使用的普通填充方塊。'),
 'Blocks':('Normal BlockItems used to fill liquid. Sponge selection is independent of this filter.','用于填充液体的普通方块物品；海绵选择不受此列表限制。','用於填充液體的普通方塊物品；海綿選擇不受此清單限制。'),
 'Placer':('Configure the shared placement queue, rotations, crystal removal and rendering.','配置放置队列、转向、阻挡水晶处理与渲染。','設定放置佇列、轉向、阻擋水晶處理與繪製。'),
 'Range':('Maximum visible click or attack reach.','目标可见时允许的最大点击或攻击距离。','目標可見時允許的最大點擊或攻擊距離。'),
 'WallRange':('Reach allowed without an unobstructed world ray.','无需世界射线无遮挡时允许的交互距离。','無須世界射線暢通時允許的互動距離。'),
 'Cooldown':('Random ticks to wait after a scheduled placement attempt. The cooldown is assigned after the previous wait reaches zero.','计划放置尝试后随机等待的游戏刻数；前一次等待归零后才设置新的冷却。','計畫放置嘗試後隨機等待的遊戲刻數；前次等待歸零後才設定新的冷卻。'),
 'Swing':('Choose whether placement swings are visible on the client.','选择是否在本地显示放置挥手。','選擇是否在本機顯示放置揮手。'),
 'SwingMode':('Choose where crystal attack swings are visible.','选择水晶攻击挥手在哪一端显示。','選擇水晶攻擊揮手在哪一端顯示。'),
 'ConstructFailResult':('Use the planned hit if target-shape click verification fails.','目标形状点击验证失败时使用预先计划的点击结果。','目標形狀點擊驗證失敗時使用預先計畫的點擊結果。'),
 'Sneak':('Random movement-input ticks to sneak after selecting an interactable support block.','选择可交互的承载方块后，随机潜行这些移动输入刻。','選擇可互動的承載方塊後，隨機潛行這些移動輸入刻。'),
 'Ignore':('Allow placement despite the selected container-screen or item-use restriction.','忽略所选的容器界面或物品使用限制，继续放置。','忽略所選的容器介面或物品使用限制，繼續放置。'),
 'SlotResetDelay':('Random ticks before restoring the real selected hotbar slot after silent selection.','静默选择快捷栏后，随机等待这些游戏刻再恢复真实选中槽位。','靜默選取快捷列後，隨機等待這些遊戲刻再復原實際選取欄位。'),
 'RotationMode':('Choose normal managed rotations or placement and attack without changing the managed view.','选择正常托管转向，或在不改变托管视角的情况下操作。','選擇正常受管轉向，或在不改變受管視角的情況下操作。'),
 'PostMove':('Run the deferred action after normal movement packets when the global rotation executor permits it; otherwise defer to a later tick.','全局转向执行器允许时，在普通移动发包后执行延迟动作，否则留到后续游戏刻。','全域轉向執行器允許時，在普通移動發包後執行延遲動作，否則留至後續遊戲刻。'),
 'SendRotationPacket':('Send a normalized rotation packet immediately before the deferred action.','执行延迟动作前发送归一化后的转向数据包。','執行延遲動作前傳送正規化後的轉向封包。'),
 'Placements':('Number of placement tasks scheduled in one operation without managed rotations.','不使用托管转向时，每次操作计划的放置任务数量。','不使用受管轉向時，每次操作計畫的放置工作數量。'),
 'DestroyCrystals':('Destroy reachable end crystals blocking queued positions unless the estimated explosion would be lethal.','处理阻挡队列位置且可攻击的末地水晶；预计爆炸会致死时放弃目标。','處理阻擋佇列位置且可攻擊的終界水晶；預計爆炸會致命時放棄目標。'),
 'Enabled':('Enable this feature or renderer.','启用此功能或渲染器。','啟用此功能或繪製器。'),
 'Delay':('Minimum milliseconds between crystal attack attempts.','两次水晶攻击尝试之间的最短毫秒间隔。','兩次水晶攻擊嘗試之間的最短毫秒間隔。'),
 'Instant':('Immediately execute when the active mode can finish; otherwise use the deferred executor.','当前模式已能完成时立即执行，否则使用延迟执行器。','目前模式已能完成時立即執行，否則使用延遲執行器。'),
 'IgnoreOpenInventory':('Allow the managed crystal rotation request while a container screen is open.','打开容器界面时仍允许托管水晶转向请求。','開啟容器介面時仍允許受管水晶轉向請求。'),
 'TargetRendering':('Render queued positions; retained queue entries keep their insertion order.','渲染待放置的位置；保留的队列项保持插入顺序。','繪製待放置位置；保留的佇列項目維持插入順序。'),
 'PlacedRendering':('Animate successful placement or item-use positions using their real outline bounds.','使用真实轮廓范围，为放置或物品使用成功的位置播放动画。','使用實際輪廓範圍，為放置或物品使用成功的位置播放動畫。'),
 'Clump':('Cull shared faces and edges between adjacent rendered blocks.','剔除相邻渲染方块之间共用的面和边。','剔除相鄰繪製方塊之間共用的面與邊。'),
 'StartSize':('Initial scale of an entering outline.','轮廓进入动画的初始缩放比例。','輪廓進入動畫的初始縮放比例。'),
 'StartCurve':('Easing curve for the entering scale animation.','进入缩放动画使用的缓动曲线。','進入縮放動畫使用的緩動曲線。'),
 'EndSize':('Final scale of a leaving outline.','轮廓退出动画的最终缩放比例。','輪廓離開動畫的最終縮放比例。'),
 'EndCurve':('Easing curve for the leaving scale animation.','退出缩放动画使用的缓动曲线。','離開縮放動畫使用的緩動曲線。'),
 'FadeInCurve':('Easing curve for entering opacity.','进入透明度动画使用的缓动曲线。','進入透明度動畫使用的緩動曲線。'),
 'FadeOutCurve':('Easing curve for leaving opacity.','退出透明度动画使用的缓动曲线。','離開透明度動畫使用的緩動曲線。'),
 'InTime':('Duration in milliseconds of the entering animation.','进入动画的毫秒时长。','進入動畫的毫秒長度。'),
 'OutTime':('Duration in milliseconds of the leaving animation.','退出动画的毫秒时长。','離開動畫的毫秒長度。'),
 'Color':('ARGB color of filled faces.','填充面的 ARGB 颜色。','填滿面的 ARGB 色彩。'),
 'OutlineColor':('ARGB color of outline edges.','轮廓边线的 ARGB 颜色。','輪廓邊線的 ARGB 色彩。'),
}
OPTION_TEXT={
 'Water':('Fill replaceable water source blocks.','填充可替换的水源方块。','填充可取代的水源方塊。'),'Lava':('Fill replaceable lava source blocks.','填充可替换的熔岩源方块。','填充可取代的熔岩源方塊。'),
 'Random':('Shuffle newly found source positions.','随机打乱新发现的液体源位置。','隨機打亂新發現的液體源位置。'),'CloserFirst':('Append nearer sources first.','优先追加较近的液体源。','優先附加較近的液體源。'),'FurtherFirst':('Append farther sources first.','优先追加较远的液体源。','優先附加較遠的液體源。'),'BottomTop':('Append lower sources first, preserving scan order on equal heights.','优先追加较低的液体源，同高度保持扫描顺序。','優先附加較低的液體源，同高度維持掃描順序。'),'TopBottom':('Append higher sources first, preserving scan order on equal heights.','优先追加较高的液体源，同高度保持扫描顺序。','優先附加較高的液體源，同高度維持掃描順序。'),
 'Whitelist':('Use normal filler blocks listed in Blocks.','仅使用方块列表内的普通填充方块。','僅使用方塊清單內的普通填充方塊。'),'Blacklist':('Use normal filler blocks absent from Blocks.','使用方块列表以外的普通填充方块。','使用方塊清單以外的普通填充方塊。'),
 'OpenInventory':('Allow placement with a container screen open.','打开容器界面时仍允许放置。','開啟容器介面時仍允許放置。'),'UsingItem':('Allow placement while using an item.','使用物品时仍允许放置。','使用物品時仍允許放置。'),
 'Normal':('Use the shared RotationManager and wait for its completion check.','使用共享转向管理器并等待完成检查。','使用共用轉向管理器並等待完成檢查。'),'None':('Schedule actions without a managed rotation request.','计划动作时不提交托管转向请求。','計畫動作時不提交受管轉向請求。'),
 'DoNotHide':('Show swings on both client and server.','客户端与服务器均显示挥手。','用戶端與伺服器皆顯示揮手。'),'HideForBoth':('Hide swings on both client and server.','客户端与服务器均隐藏挥手。','用戶端與伺服器皆隱藏揮手。'),'HideForClient':('Hide swings on the client while sending the server swing.','本地隐藏挥手，仍向服务器发送挥手。','本機隱藏揮手，仍向伺服器傳送揮手。'),'HideForServer':('Show the local swing without sending the server swing.','本地显示挥手，不向服务器发送挥手。','本機顯示揮手，不向伺服器傳送揮手。'),
}
ROTATION_TEXT={
 '':('Configure managed placement or crystal rotations and their reset.','配置放置或水晶的托管转向及其复位。','設定放置或水晶的受管轉向及其復位。'),
 'MovementCorrection':('Choose how movement follows managed rotations: Off, Strict, Silent or ChangeLook.','选择移动如何配合托管转向：关闭、严格、静默或改变视角。','選擇移動如何配合受管轉向：關閉、嚴格、靜默或改變視角。'),
 'ResetThreshold':('Finish resetting within this many degrees of the visible view.','与可见视角的角度差不高于此值时结束复位。','與可見視角的角度差不高於此值時結束復位。'),
 'TicksUntilReset':('Keep the request for this many ticks before resetting.','保留请求这些游戏刻后开始复位。','保留請求這些遊戲刻後開始復位。'),
 'AngleSmooth':('Choose the smoothing algorithm; inactive algorithm values remain saved.','选择转向平滑算法；非活动算法的设置也会保存。','選擇轉向平滑演算法；未啟用演算法的設定也會儲存。'),
 'AngleSmooth.Linear':('Limit horizontal and vertical angle changes independently.','分别限制水平与垂直角度变化。','分別限制水平與垂直角度變化。'),
 'AngleSmooth.Sigmoid':('Scale angle changes with an S-shaped speed curve.','使用 S 形速度曲线缩放角度变化。','使用 S 形速度曲線縮放角度變化。'),
 'AngleSmooth.Acceleration':('Smooth rotation through bounded changes in turning speed.','通过限制转向速度的变化来平滑转向。','透過限制轉向速度的變化來平滑轉向。'),
 'AngleSmooth.Acceleration.DynamicAccel':('Configure entity-aware acceleration; these requests supply only angles.','配置基于实体的加速度；此处请求仅提供角度。','設定基於實體的加速度；此處請求僅提供角度。'),
 'AngleSmooth.Acceleration.AccelerationError':('Configure random errors proportional to acceleration.','配置与加速度成比例的随机误差。','設定與加速度成比例的隨機誤差。'),
 'AngleSmooth.Acceleration.ConstantError':('Configure random fixed-angle errors.','配置随机固定角度误差。','設定隨機固定角度誤差。'),
 'AngleSmooth.Acceleration.SigmoidDeceleration':('Configure S-shaped acceleration deceleration.','配置 S 形加速度减速。','設定 S 形加速度減速。'),
 'MovementCorrection.Off':('Keep movement input unchanged by the managed rotation.','托管转向不修正移动输入。','受管轉向不修正移動輸入。'),
 'MovementCorrection.Strict':('Match movement strictly to the managed rotation.','严格按托管转向修正移动。','嚴格依受管轉向修正移動。'),
 'MovementCorrection.Silent':('Correct movement while preserving the visible view.','在保持可见视角的同时修正移动。','在維持可見視角的同時修正移動。'),
 'MovementCorrection.ChangeLook':('Make the visible view follow the managed rotation.','让可见视角跟随托管转向。','讓可見視角跟隨受管轉向。'),
}
EASINGS={
 'Linear':(('线性','線性'),('Use linear progress.','使用线性进度。','使用線性進度。')),
 'QuadIn':(('二次渐入','二次漸入'),('Use quadratic ease-in progress.','使用二次方渐入进度。','使用二次方漸入進度。')),
 'QuadOut':(('二次渐出','二次漸出'),('Use quadratic ease-out progress.','使用二次方渐出进度。','使用二次方漸出進度。')),
 'QuadInOut':(('二次渐入渐出','二次漸入漸出'),('Use quadratic ease-in-out progress.','使用二次方渐入渐出进度。','使用二次方漸入漸出進度。')),
 'ExponentialIn':(('指数渐入','指數漸入'),('Use exponential ease-in progress.','使用指数渐入进度。','使用指數漸入進度。')),
 'ExponentialOut':(('指数渐出','指數漸出'),('Use exponential ease-out progress.','使用指数渐出进度。','使用指數漸出進度。')),
 'None':(('无缓动','無緩動'),('Immediately use the final factor.','立即使用最终动画系数。','立即使用最終動畫係數。')),
}
def camel(path):return '.'.join(p[:1].lower()+p[1:] for p in path.split('.'))
def rot(path,text):ROTATION_TEXT[path]=text
for mode in ['Linear','Sigmoid']:
 for axis,cn,tw in [('Horizontal','水平','水平'),('Vertical','垂直','垂直')]:
  rot(f'AngleSmooth.{mode}.{axis}TurnSpeed',(f'Random {axis.lower()} turn limit per tick.',f'每游戏刻随机采用的{cn}转向角度上限。',f'每遊戲刻隨機採用的{tw}轉向角度上限。'))
for branch in ['AngleSmooth.Sigmoid','AngleSmooth.Acceleration.SigmoidDeceleration']:
 rot(branch+'.Steepness',('Steepness of the S-shaped curve.','S 形曲线的陡峭程度。','S 形曲線的陡峭程度。'));rot(branch+'.Midpoint',('Midpoint of the S-shaped curve.','S 形曲线的中点。','S 形曲線的中點。'))
for field,en,cn,tw in [
 ('YawAcceleration','Maximum random horizontal turn-speed change per tick.','每游戏刻水平转向速度变化量的随机上限。','每遊戲刻水平轉向速度變化量的隨機上限。'),
 ('PitchAcceleration','Maximum random vertical turn-speed change per tick.','每游戏刻垂直转向速度变化量的随机上限。','每遊戲刻垂直轉向速度變化量的隨機上限。'),
 ('DynamicAccel.Enabled','Enable entity-aware acceleration; these rotation-only requests provide no target entity.','启用基于实体的加速度；此处仅有角度请求，不提供目标实体。','啟用基於實體的加速度；此處僅有角度請求，不提供目標實體。'),
 ('DynamicAccel.CoefDistance','Target distance acceleration coefficient; these requests use zero target distance.','目标距离的加速度系数；此处请求使用零目标距离。','目標距離的加速度係數；此處請求使用零目標距離。'),
 ('DynamicAccel.YawCrosshairAccel','Horizontal acceleration when aiming at the target entity; unused without a target entity.','准星对准目标实体时的水平加速度；无目标实体时不使用。','準星對準目標實體時的水平加速度；無目標實體時不使用。'),
 ('DynamicAccel.PitchCrosshairAccel','Vertical acceleration when aiming at the target entity; unused without a target entity.','准星对准目标实体时的垂直加速度；无目标实体时不使用。','準星對準目標實體時的垂直加速度；無目標實體時不使用。'),
 ('AccelerationError.Enabled','Add random error proportional to turn acceleration.','添加与转向加速度成比例的随机误差。','加入與轉向加速度成比例的隨機誤差。'),
 ('AccelerationError.YawAccelError','Maximum proportional horizontal acceleration error.','水平转向加速度随机比例误差的最大值。','水平轉向加速度隨機比例誤差的最大值。'),
 ('AccelerationError.PitchAccelError','Maximum proportional vertical acceleration error.','垂直转向加速度随机比例误差的最大值。','垂直轉向加速度隨機比例誤差的最大值。'),
 ('ConstantError.Enabled','Add random fixed-angle errors to each turn.','为每次转向添加随机固定角度误差。','為每次轉向加入隨機固定角度誤差。'),
 ('ConstantError.YawConstantError','Maximum horizontal fixed-angle error.','水平转向随机固定角度误差的最大值。','水平轉向隨機固定角度誤差的最大值。'),
 ('ConstantError.PitchConstantError','Maximum vertical fixed-angle error.','垂直转向随机固定角度误差的最大值。','垂直轉向隨機固定角度誤差的最大值。'),
 ('SigmoidDeceleration.Enabled','Scale acceleration with an S-shaped curve as the remaining angle changes.','按剩余角度对应的 S 形曲线缩放加速度。','依剩餘角度對應的 S 形曲線縮放加速度。'),
]:rot('AngleSmooth.Acceleration.'+field,(en,cn,tw))
def entries():
 texts={'description':('Fill nearby replaceable liquid sources using normal blocks or sponge planning.','使用普通方块或海绵规划，填充附近可替换的液体源。','使用普通方塊或海綿規劃，填充附近可取代的液體源。')};scoped={}
 def add(path,description):
  suffix=camel(path);texts[suffix+'.description']=description
  if path:scoped[suffix]=LABELS[path.rsplit('.',1)[-1]]
 def walk(nodes,prefix=''):
  for node in nodes:
   name=node['name'];path=prefix+'.'+name if prefix else name;kind=node['kind']
   if kind=='ROTATIONS':
    for suffix,description in ROTATION_TEXT.items():add(path+('.'+suffix if suffix else ''),description)
    for option in ['Off','Strict','Silent','ChangeLook']:scoped[camel(path+'.MovementCorrection')+'.'+option]=LABELS[option]
    for suffix in ['DynamicAccel','AccelerationError','ConstantError','SigmoidDeceleration']:
     scoped[camel(path+'.AngleSmooth.Acceleration.'+suffix)]=LABELS[suffix]
    continue
   add(path,DESCRIPTIONS.get(name) or OPTION_TEXT[name])
   for option in node.get('options',[]):
    scoped[camel(path)+'.'+option]=LABELS[option];texts[camel(path+'.'+option)+'.description']=OPTION_TEXT[option]
    if name=='RotationMode' and option=='None':scoped[camel(path)+'.None']=('不转向','不轉向')
   walk(node.get('children',[]),path);walk(node.get('modes',[]),path)
   if kind=='PLACEMENT':
    for leaf in ['Enabled','Clump','StartSize','StartCurve','EndSize','EndCurve','FadeInCurve','FadeOutCurve','InTime','OutTime','Color','OutlineColor']:add(path+'.'+leaf,DESCRIPTIONS[leaf])
    for leaf in ['StartCurve','EndCurve','FadeInCurve','FadeOutCurve']:
     for option,(label,description) in EASINGS.items():scoped[camel(path+'.'+leaf)+'.'+option]=label;texts[camel(path+'.'+leaf+'.'+option)+'.description']=description
 walk(json.loads((HERE/'engine/src/config/liquid_filler_schema.json').read_text(encoding='utf8')))
 return texts,scoped
def update(folder:Path):
 texts,scoped=entries()
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  values={'':LABELS['LiquidFiller'][index-1],**{key:value[index] for key,value in texts.items()}}
  put_module(data,PREFIX,'LiquidFiller',values,{key:value[0] for key,value in texts.items()})
  labels=data.setdefault('labels',{})
  for key,value in LABELS.items():labels.setdefault(key,value[index-1])
  for key,value in scoped.items():labels[PREFIX+'.'+key]=value[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(parser.parse_args().folder)
