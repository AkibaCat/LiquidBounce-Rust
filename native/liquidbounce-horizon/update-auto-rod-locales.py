#!/usr/bin/env python3
"""AutoRod display localization; identifiers and stored enum values stay stable."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.autoRod'
LABELS={
 'AutoRod':('自动鱼竿','自動釣竿'),'GravityType':('重力模式','重力模式'),'Linear':('直线瞄准','直線瞄準'),'Projectile':('弹道瞄准','彈道瞄準'),
 'ScanExtraRange':('额外扫描距离','額外掃描距離'),'MaxEnemiesNearby':('附近敌人上限','附近敵人上限'),'MinTargetHealth':('目标最低生命值','目標最低生命值'),
 'Requires':('启用条件','啟用條件'),'Click':('攻击键按下','攻擊鍵按下'),'MouseMovement':('近期鼠标转动','近期滑鼠轉動'),'Weapon':('主手持有武器','主手持有武器'),
 'VanillaName':('没有自定义名称','沒有自訂名稱'),'NotBreaking':('没有破坏方块','沒有破壞方塊'),'Ignore':('暂停条件','暫停條件'),'HoldingConsumable':('持有可消耗物品','持有可消耗物品'),
 'HoldingItemsForIgnore':('持有时暂停的物品','持有時暫停的物品'),'AimOffThreshold':('瞄准偏差阈值','瞄準偏差門檻'),'HitTimeout':('收竿超时','收竿逾時'),
 'PullOnOutOfRange':('超出范围时收竿','超出範圍時收竿'),'SlotResetDelay':('槽位复原延迟','欄位復原延遲'),
 'AimPoint':('瞄准点','瞄準點'),'ExemptBoxParts':('排除身体部位','排除身體部位'),'Head':('头部','頭部'),'Body':('躯干','軀幹'),'Feet':('脚部','腳部'),
 'ExemptBestHitVector':('排除最佳命中点附近','排除最佳命中點附近'),'Vertical':('垂直范围','垂直範圍'),'Horizontal':('水平范围','水平範圍'),
 'Gaussian':('高斯偏移','高斯偏移'),'YawOffset':('水平偏移系数','水平偏移係數'),'PitchOffset':('垂直偏移系数','垂直偏移係數'),'Tolerance':('接近容差','接近容差'),
 'Dynamic':('动态调整','動態調整'),'YawFactor':('水平速度系数','水平速度係數'),'PitchFactor':('垂直速度系数','垂直速度係數'),
 'Lazy':('惰性跟踪','惰性追蹤'),'Threshold':('更新距离阈值','更新距離門檻'),
}
TEXTS={}
def text(path,en,cn,tw):TEXTS[path+'.description'if path else'description']=(en,cn,tw)
text('','Automatically cast and retrieve a fishing rod against selected combat targets.','自动向选定战斗目标抛出鱼竿，并在命中、静止、超时或超出范围时收竿。','自動向選定戰鬥目標拋出釣竿，並在命中、靜止、逾時或超出範圍時收竿。')
text('gravityType','Choose direct point tracking or a gravity-and-drag projectile solution.','选择直线瞄准点跟踪，或包含重力与阻力的弹道解算。','選擇直線瞄準點追蹤，或包含重力與阻力的彈道解算。')
text('range','Minimum and maximum player-to-target position distance for rod use.','使用鱼竿时玩家与目标位置之间允许的最小和最大距离。','使用釣竿時玩家與目標位置之間允許的最小和最大距離。')
text('scanExtraRange','Random extension of the maximum range, sampled again after each successful cast.','最大作用距离的随机扩展值；每次成功抛竿后重新抽取。','最大作用距離的隨機擴展值；每次成功拋竿後重新抽取。')
text('maxEnemiesNearby','Maximum number of globally eligible targets; zero disables this limit.','可攻击目标的数量上限；按目标筛选条件统计整个已加载世界，零表示不限制。','可攻擊目標的數量上限；按目標篩選條件統計整個已載入世界，零表示不限制。')
text('minHealth','Require the player health to be strictly greater than this value.','玩家生命值必须严格高于此值。','玩家生命值必須嚴格高於此值。')
text('minTargetHealth','Require target health to be strictly greater than this value.','目标生命值必须严格高于此值；支持识别名称下方的生命值计分板。','目標生命值必須嚴格高於此值；支援辨識名稱下方的生命值計分板。')
text('requires','All selected requirements must hold before aiming or starting a new rod sequence.','开始瞄准或新的鱼竿操作前，必须满足所有选中的启用条件。','開始瞄準或新的釣竿操作前，必須滿足所有選取的啟用條件。')
text('requires.click','Require the physical attack binding to be held, or its mouse button to have been pressed within 250 ms.','要求实际攻击键保持按下，或对应鼠标按键在最近二百五十毫秒内按下过。','要求實際攻擊鍵保持按下，或對應滑鼠按鍵在最近二百五十毫秒內按下過。')
text('requires.mouseMovement','Require nonzero physical mouse turn input within the previous 100 ms.','要求最近一百毫秒内出现非零的实际鼠标转动输入。','要求最近一百毫秒內出現非零的實際滑鼠轉動輸入。')
text('requires.weapon','Require a sword, a modern-combat axe, a mace, or an item enchanted with Knockback in the main hand.','要求主手持剑、现代战斗版本的斧、重锤，或带有击退附魔的物品。','要求主手持劍、現代戰鬥版本的斧、重錘，或帶有擊退附魔的物品。')
text('requires.vanillaName','Require the main-hand item to have no custom name component.','要求主手物品没有自定义名称组件。','要求主手物品沒有自訂名稱元件。')
text('requires.notBreaking','Require the interaction manager not to be destroying a block.','要求交互管理器当前没有正在破坏的方块。','要求互動管理器目前沒有正在破壞的方塊。')
text('ignore','Pause aiming and new rod sequences when any selected condition is true.','任一选中暂停条件成立时，暂停瞄准和开始新的鱼竿操作。','任一選取暫停條件成立時，暫停瞄準和開始新的釣竿操作。')
text('ignore.openInventory','Pause when a container screen or server-side inventory is open.','容器界面或服务器侧物品栏处于打开状态时暂停。','容器介面或伺服器端物品欄處於開啟狀態時暫停。')
text('ignore.usingItem','Pause while the player is actually using an item.','玩家实际处于使用物品状态时暂停。','玩家實際處於使用物品狀態時暫停。')
text('ignore.holdingConsumable','Pause when either hand has the consumable component.','任意一只手持有带可消耗组件的物品时暂停。','任意一隻手持有帶可消耗元件的物品時暫停。')
text('holdingItemsForIgnore','Never start rod use while the main hand holds one of these registry items.','主手持有列表中的物品时不开始鱼竿操作；默认包括弓、弩、三叉戟、火焰弹及末影珍珠。','主手持有清單中的物品時不開始釣竿操作；預設包括弓、弩、三叉戟、火焰彈及終界珍珠。')
text('aimOffThreshold','Maximum angle between the server direction and the freshly calculated rod aim before casting.','抛竿前，服务器朝向与本次重新计算的鱼竿瞄准方向之间允许的最大夹角。','拋竿前，伺服器朝向與本次重新計算的釣竿瞄準方向之間允許的最大夾角。')
text('swingMode','Show the predicted interaction swing or hide it from the local client.','显示预测的交互挥手动画，或仅对本地客户端隐藏。','顯示預測的互動揮手動畫，或僅對本機用戶端隱藏。')
text('hitTimeout','Maximum ticks to wait before retrieving the hook.','收回鱼钩前最多等待的游戏刻数。','收回浮標前最多等待的遊戲刻數。')
text('pullOnOutOfRange','Retrieve the rod if the captured target leaves the range selected for this cast.','本次操作捕获的目标离开抛竿时确定的距离范围后立即收竿。','本次操作捕獲的目標離開拋竿時確定的距離範圍後立即收竿。')
text('slotResetDelay','Random ticks to retain the rod slot after pulling; offhand use does not select a hotbar slot.','收竿后保留鱼竿槽位的随机游戏刻数；副手操作不切换快捷栏。','收竿後保留釣竿欄位的隨機遊戲刻數；副手操作不切換快捷列。')
text('cooldown','Random ticks after pulling before another rod sequence can start.','收竿后，下一次鱼竿操作开始前随机等待的游戏刻数。','收竿後，下一次釣竿操作開始前隨機等待的遊戲刻數。')
text('aimPoint','Project points onto the predicted target hitbox and process Delay, Lazy, then Gaussian offsets.','将瞄准点投影到预测目标碰撞箱，依次应用延迟、惰性跟踪与高斯偏移。','將瞄準點投影到預測目標碰撞箱，依次套用延遲、惰性追蹤與高斯偏移。')
text('aimPoint.exemptBoxParts','Remove selected thirds of the target hitbox from point candidates.','从候选瞄准点中排除碰撞箱上、中、下三等分所对应的身体部位。','從候選瞄準點中排除碰撞箱上、中、下三等分所對應的身體部位。')
text('aimPoint.exemptBestHitVector','Exclude points around the nearest original hit vector using separate vertical and horizontal tolerances.','按独立的垂直和水平范围排除原始最近命中点附近的候选点。','按獨立的垂直和水平範圍排除原始最近命中點附近的候選點。')
text('aimPoint.exemptBestHitVector.enabled','Enable exclusion around the nearest hit vector.','启用最佳命中点附近的候选点排除。','啟用最佳命中點附近的候選點排除。')
text('aimPoint.exemptBestHitVector.vertical','Vertical exclusion distance from the nearest hit vector.','距离最佳命中点的垂直排除范围。','距離最佳命中點的垂直排除範圍。')
text('aimPoint.exemptBestHitVector.horizontal','Horizontal exclusion radius around the nearest hit vector.','最佳命中点周围的水平排除半径。','最佳命中點周圍的水平排除半徑。')
text('aimPoint.gaussian','Move a persistent random offset toward successive Gaussian targets.','让持续保存的随机偏移逐步移向新的高斯分布目标偏移。','讓持續儲存的隨機偏移逐步移向新的高斯分布目標偏移。')
text('aimPoint.gaussian.enabled','Enable Gaussian offset processing.','启用高斯偏移处理。','啟用高斯偏移處理。')
text('aimPoint.gaussian.yawOffset','Random horizontal multiplier for the Gaussian X and Z offsets.','高斯 X、Z 轴偏移的随机水平乘数。','高斯 X、Z 軸偏移的隨機水平乘數。')
text('aimPoint.gaussian.pitchOffset','Random vertical multiplier for the Gaussian Y offset.','高斯 Y 轴偏移的随机垂直乘数。','高斯 Y 軸偏移的隨機垂直乘數。')
text('aimPoint.gaussian.chance','Probability of choosing another Gaussian target once the previous one is close enough.','接近上一个目标偏移后，重新选择高斯目标偏移的概率。','接近上一個目標偏移後，重新選擇高斯目標偏移的機率。')
text('aimPoint.gaussian.speed','Independently sampled interpolation factor for each offset axis.','每个偏移轴分别抽取的插值速度系数。','每個偏移軸分別抽取的插值速度係數。')
text('aimPoint.gaussian.tolerance','Per-axis distance treated as having reached the target offset.','每个坐标轴上视为已接近目标偏移的距离容差。','每個座標軸上視為已接近目標偏移的距離容差。')
text('aimPoint.gaussian.dynamic','Configure the entity-aware Gaussian helper; point-box processing retains the upstream non-entity path.','配置支持实体参数的高斯处理器；瞄准点处理保持原版的非实体输入路径。','設定支援實體參數的高斯處理器；瞄準點處理保留原版的非實體輸入路徑。')
text('aimPoint.gaussian.dynamic.enabled','Enable entity-aware dynamic adjustment when a living-entity input is supplied.','传入生物实体时允许按实体状态动态调整。','傳入生物實體時允許按實體狀態動態調整。')
text('aimPoint.gaussian.dynamic.hurtTime','Minimum input-entity hurt time for dynamic adjustment.','动态调整要求输入实体具有的最低受伤计时。','動態調整要求輸入實體具有的最低受傷計時。')
text('aimPoint.gaussian.dynamic.yawFactor','Add player horizontal speed multiplied by this factor to horizontal offset strength.','将玩家水平速度乘以此系数，加入水平偏移强度。','將玩家水平速度乘以此係數，加入水平偏移強度。')
text('aimPoint.gaussian.dynamic.pitchFactor','Add player horizontal speed multiplied by this factor to vertical offset strength.','将玩家水平速度乘以此系数，加入垂直偏移强度。','將玩家水平速度乘以此係數，加入垂直偏移強度。')
text('aimPoint.gaussian.dynamic.speed','Per-axis interpolation factor used by active dynamic adjustment.','动态调整生效时每个坐标轴使用的插值速度系数。','動態調整生效時每個座標軸使用的插值速度係數。')
text('aimPoint.gaussian.dynamic.tolerance','Per-axis arrival tolerance used by active dynamic adjustment.','动态调整生效时每个坐标轴的接近目标容差。','動態調整生效時每個座標軸的接近目標容差。')
text('aimPoint.lazy','Retain a previous point until the sampled distance threshold is reached.','未达到本次抽取的距离阈值时保留上一个瞄准点。','未達到本次抽取的距離門檻時保留上一個瞄準點。')
text('aimPoint.lazy.enabled','Enable lazy point updates.','启用瞄准点的惰性更新。','啟用瞄準點的惰性更新。')
text('aimPoint.lazy.threshold','Sample a new minimum movement distance after a point is accepted.','接受新的瞄准点后重新抽取下次更新所需的最小移动距离。','接受新的瞄準點後重新抽取下次更新所需的最小移動距離。')
text('aimPoint.delay','Delay adoption of changed point-box samples.','延迟采纳发生变化的瞄准点与包围盒。','延遲採納發生變化的瞄準點與包圍盒。')
text('aimPoint.delay.enabled','Enable point-delay processing before lazy and Gaussian processing.','在惰性跟踪与高斯偏移前启用瞄准点延迟。','在惰性追蹤與高斯偏移前啟用瞄準點延遲。')
text('aimPoint.delay.delay','Random number of changed samples before the stored point is replaced.','储存的瞄准点被替换前需要经过的随机变化采样次数。','儲存的瞄準點被替換前需要經過的隨機變化取樣次數。')
def update(folder):
 bow=runpy.run_path(str(HERE/'update-auto-bow-locales.py'))['TEXTS']
 rendering=runpy.run_path(str(HERE/'update-target-renderer-locales.py'))['update']
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoRod'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()});english={k:v[0]for k,v in TEXTS.items()}
  for k,v in bow.items():
   if k.startswith('bowAimbot.target.')or k=='bowAimbot.target.description':
    key=k.removeprefix('bowAimbot.');entries[key]=v[index];english[key]=v[0]
  put_module(data,PREFIX,'AutoRod',entries,english);data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
   for k,v in list(data.get(field,{}).items()):
    if k.startswith(old):data[field][k.replace(old,PREFIX+'.rotations')]=v
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
  rendering(locale,path,PREFIX+'.targetRendering')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');args=p.parse_args();update(args.folder)
