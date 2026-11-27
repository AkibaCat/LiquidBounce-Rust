#!/usr/bin/env python3
"""KillAura display text; keeps source configuration identifiers and enum tags."""
from pathlib import Path
import argparse
import json
import runpy
from display_locale_schema import put_module

HERE = Path(__file__).resolve().parent
PREFIX = 'liquidbounce.module.killAura'
LABELS = {
    'KillAura': ('杀戮光环', '殺戮光環'),
    'ItemCooldown': ('物品攻击冷却', '物品攻擊冷卻'),
    'Minimum': ('最低冷却进度', '最低冷卻進度'),
    'IgnoreOnShieldBreak': ('破盾时忽略冷却', '破盾時忽略冷卻'),
    'IgnoreOnMaceSmash': ('重锤猛击时忽略冷却', '重錘猛擊時忽略冷卻'),
    'IgnoreWhenExitingRange': ('即将离开距离时忽略冷却', '即將離開距離時忽略冷卻'),
    'RangeIncrease': ('攻击距离增量', '攻擊距離增量'),
    'ThroughWallsRange': ('遮挡攻击距离', '遮擋攻擊距離'),
    'ScanRangeIncrease': ('扫描距离增量', '掃描距離增量'),
    'IgnoreShield': ('忽略盾牌阻挡', '忽略盾牌阻擋'),
    'Raycast': ('攻击射线目标', '攻擊射線目標'),
    'Enemy': ('仅敌对目标', '僅敵對目標'),
    'Criticals': ('暴击选择', '暴擊選擇'),
    'Smart': ('智能', '智慧'),
    'KeepSprint': ('保持疾跑', '保持疾跑'),
    'IgnoreOpenInventory': ('打开物品栏时继续', '開啟物品欄時繼續'),
    'SimulateInventoryClosing': ('模拟关闭物品栏', '模擬關閉物品欄'),
    'RotationTiming': ('转向时机', '轉向時機'),
    'Snap': ('攻击瞬间转向', '攻擊瞬間轉向'),
    'OnTick': ('攻击游戏刻转向', '攻擊遊戲刻轉向'),
    'ThroughWalls': ('允许向遮挡目标转向', '允許向遮擋目標轉向'),
    'LazyRotation': ('保留已命中朝向', '保留已命中朝向'),
    'AutoBlocking': ('自动格挡', '自動格擋'),
    'BlockMode': ('格挡模式', '格擋模式'),
    'Basic': ('直接使用', '直接使用'),
    'Interact': ('先交互再使用', '先互動再使用'),
    'Fake': ('模拟格挡姿态', '模擬格擋姿態'),
    'SimulateVanillaUse': ('模拟原版使用顺序', '模擬原版使用順序'),
    'UnblockMode': ('解除格挡方式', '解除格擋方式'),
    'StopUsingItem': ('停止使用物品', '停止使用物品'),
    'ChangeSlot': ('切换快捷栏槽位', '切換快捷列欄位'),
    'SwapHand': ('交换双手', '交換雙手'),
    'Reblock': ('重新格挡延迟', '重新格擋延遲'),
    'PauseOnUnblock': ('解除格挡后攻击暂停', '解除格擋後攻擊暫停'),
    'Blink': ('移动包暂存刻数', '移動封包暫存刻數'),
    'PrioritizeBlocking': ('优先格挡', '優先格擋'),
    'OnScanRange': ('扫描距离内格挡', '掃描距離內格擋'),
    'OnlyWhenInDanger': ('仅有受击危险时', '僅有受擊危險時'),
    'ForceActiveRange': ('强制格挡距离', '強制格擋距離'),
    'AssumeShield': ('兼容旧版盾牌', '相容舊版盾牌'),
    'FailSwing': ('未命中挥手', '未命中揮手'),
    'AdditionalRange': ('额外挥手距离', '額外揮手距離'),
    'NotifyWhenFail': ('未命中反馈', '未命中回饋'),
    'Sound': ('声音', '音效'),
    'Volume': ('音量', '音量'),
    'FightBot': ('自动战斗寻路', '自動戰鬥尋路'),
    'OpponentRange': ('对手威胁距离', '對手威脅距離'),
    'DangerousYaw': ('危险朝向夹角', '危險朝向夾角'),
    'RunawayOnCooldown': ('等待冷却时后撤', '等待冷卻時後撤'),
    'TargetFilter': ('寻路目标筛选', '尋路目標篩選'),
    'NotWhenVoid': ('排除下方无方块的目标', '排除下方無方塊的目標'),
    'Leader': ('跟随领队', '跟隨領隊'),
    'Username': ('玩家名称', '玩家名稱'),
    'RangeIndicator': ('攻击距离指示', '攻擊距離指示'),
    'ColorMode': ('颜色模式', '色彩模式'),
    'IdleColor': ('待机颜色', '待機色彩'),
    'ActiveColor': ('攻击颜色', '攻擊色彩'),
    'PulseAnimation': ('脉冲动画', '脈衝動畫'),
    'PulseSpeed': ('脉冲速度', '脈衝速度'),
    'PulseIntensity': ('脉冲强度', '脈衝強度'),
    'FadeAnimation': ('淡入淡出动画', '淡入淡出動畫'),
    'FadeSpeed': ('淡入淡出速度', '淡入淡出速度'),
    'WallRangeColor': ('遮挡距离颜色', '遮擋距離色彩'),
    'ScanRangeColor': ('扫描距离颜色', '掃描距離色彩'),
    'OpponentRangeColor': ('对手威胁距离颜色', '對手威脅距離色彩'),
    'HideWhenDead': ('死亡时隐藏', '死亡時隱藏'),
    'HideWhenSpectator': ('旁观时隐藏', '旁觀時隱藏'),
    'HideInVehicle': ('乘坐时隐藏', '乘坐時隱藏'),
    'RespectInventorySetting': ('遵守物品栏设置', '遵守物品欄設定'),
}
TEXTS = {}


def text(path, en, cn, tw):
    key = '.'.join(part[:1].lower() + part[1:] for part in path.split('.'))
    TEXTS[(key + '.description') if key else 'description'] = (en, cn, tw)


text('', 'Automatically aim and attack eligible enemies using the configured click schedule.', '按选定目标、瞄准设置与点击计划自动攻击周围敌人。', '按選定目標、瞄準設定與點擊計畫自動攻擊周圍敵人。')
text('Clicker', 'Plan attacks and coordinate presses with other users of the attack binding.', '规划攻击时间，并与使用同一攻击键的玩家输入和其他点击器协调。', '規劃攻擊時間，並與使用同一攻擊鍵的玩家輸入和其他點擊器協調。')
text('Clicker.ItemCooldown', 'Require the sampled attack cooldown unless an enabled exception applies.', '攻击冷却达到本次抽取的进度后才攻击，或满足选中的例外条件。', '攻擊冷卻達到本次抽取的進度後才攻擊，或滿足選取的例外條件。')
text('Clicker.ItemCooldown.Minimum', 'Random minimum attack cooldown progress; 1.0 means a fully charged attack.', '随机抽取最低攻击冷却进度；1.0 表示完全蓄满。', '隨機抽取最低攻擊冷卻進度；1.0 表示完全蓄滿。')
text('Clicker.ItemCooldown.IgnoreOnShieldBreak', 'Bypass cooldown when the target would block and AutoWeapon predicts a shield break.', '目标能够格挡，且自动武器预计可破盾时，允许跳过攻击冷却。', '目標能夠格擋，且自動武器預計可破盾時，允許略過攻擊冷卻。')
text('Clicker.ItemCooldown.IgnoreOnMaceSmash', 'Bypass cooldown when AutoWeapon predicts a mace smash.', '自动武器预计将触发重锤猛击时，允许跳过攻击冷却。', '自動武器預計將觸發重錘猛擊時，允許略過攻擊冷卻。')
text('Clicker.ItemCooldown.IgnoreWhenExitingRange', 'Bypass cooldown when the predicted player and target positions will leave attack range.', '预测玩家与目标即将超出可攻击距离时，允许提前攻击。', '預測玩家與目標即將超出可攻擊距離時，允許提前攻擊。')
text('Range', 'Configure visible, obstructed and target-scanning distances.', '配置可见目标、遮挡目标与目标扫描的距离。', '設定可見目標、遮擋目標與目標掃描的距離。')
text('Range.RangeIncrease', 'Add this value to the actual entity interaction attribute and held-item attack range.', '在实际实体交互属性和手持物品攻击距离基础上增加此值。', '在實際實體互動屬性和手持物品攻擊距離基礎上增加此值。')
text('Range.ThroughWallsRange', 'Maximum distance for obstructed hits, clamped to the current interaction range when changed.', '目标受到遮挡时允许命中的最大距离；修改时限制在当前交互距离内。', '目標受到遮擋時允許命中的最大距離；修改時限制在目前互動距離內。')
text('Range.ScanRangeIncrease', 'Random extension of target scan range, sampled again after attacks.', '目标扫描距离的随机扩展值；攻击后重新抽取。', '目標掃描距離的隨機擴充值；攻擊後重新抽取。')
text('Target.IgnoreShield', 'Allow selecting targets even when their shield would block this hit.', '即使目标盾牌能够格挡本次攻击，也允许选中。', '即使目標盾牌能夠格擋本次攻擊，也允許選中。')
text('Requires', 'Require every selected condition before preparing or executing combat actions.', '准备和执行战斗操作前，必须满足所有选中的条件。', '準備和執行戰鬥操作前，必須滿足所有選取的條件。')
text('Requires.EmptyHand', 'Require an empty main hand.', '要求实际主手物品堆为空。', '要求實際主手物品堆為空。')
text('Raycast', 'Choose whether the attack ray keeps its target, selects another enemy, or can select any hit entity.', '选择攻击射线保留原目标、只改选敌对目标，或允许改选命中的任何实体。', '選擇攻擊射線保留原目標、只改選敵對目標，或允許改選命中的任何實體。')
text('Criticals', 'Choose smart critical timing, always require a critical, or ignore critical timing.', '选择智能暴击时机、始终要求暴击，或忽略暴击时机。', '選擇智慧暴擊時機、始終要求暴擊，或忽略暴擊時機。')
text('KeepSprint', 'Keep sprinting when the attack path permits it.', '攻击流程允许时保持疾跑状态。', '攻擊流程允許時保持疾跑狀態。')
text('IgnoreOpenInventory', 'Continue combat while a container or server inventory is open.', '容器界面或服务器侧物品栏打开时仍继续战斗。', '容器介面或伺服器端物品欄開啟時仍繼續戰鬥。')
text('SimulateInventoryClosing', 'Send inventory-close preparation around attacks when inventory restrictions are ignored.', '允许打开物品栏时攻击的情况下，在攻击前后模拟服务器侧物品栏关闭。', '允許開啟物品欄時攻擊的情況下，在攻擊前後模擬伺服器端物品欄關閉。')
text('Rotations.RotationTiming', 'Normal prepares continuously; Snap rotates for the attack; OnTick prepares on click ticks.', '普通模式持续准备朝向；瞬间模式在攻击时转向；游戏刻模式只在点击刻准备。', '一般模式持續準備朝向；瞬間模式在攻擊時轉向；遊戲刻模式只在點擊刻準備。')
text('Rotations.ThroughWalls', 'Allow preparing a rotation towards targets hidden by blocks.', '允许向被方块遮挡的目标准备瞄准朝向。', '允許向被方塊遮擋的目標準備瞄準朝向。')
text('Rotations.LazyRotation', 'Keep the current direction when its ray can still hit the selected target.', '当前朝向的射线仍能命中选定目标时，保留此朝向。', '目前朝向的射線仍能命中選定目標時，保留此朝向。')
text('AutoBlocking', 'Use a blocking item between attacks with the selected release and reblock timing.', '在攻击间隔使用格挡物品，并遵守解除及重新格挡的时序。', '在攻擊間隔使用格擋物品，並遵守解除及重新格擋的時序。')
text('AutoBlocking.Enabled', 'Enable automatic blocking.', '启用自动格挡。', '啟用自動格擋。')
text('AutoBlocking.BlockMode', 'Basic uses the blocking item; Interact first interacts with the target; Fake only shows a blocking pose.', '直接模式使用格挡物品；交互模式先与目标交互；模拟模式只显示格挡姿态。', '直接模式使用格擋物品；互動模式先與目標互動；模擬模式只顯示格擋姿態。')
text('AutoBlocking.SimulateVanillaUse', 'Try hands in the same order as vanilla item use.', '按原版物品使用顺序尝试双手。', '按原版物品使用順序嘗試雙手。')
text('AutoBlocking.UnblockMode', 'Release by stopping use, changing slots, swapping hands, or keeping the current use.', '通过停止使用、切换槽位或交换双手解除格挡；无操作模式保留当前使用。', '透過停止使用、切換欄位或交換雙手解除格擋；無操作模式保留目前使用。')
text('AutoBlocking.Reblock', 'Random number of ticks between releasing and starting to block again.', '解除格挡后，再次开始格挡前随机等待的游戏刻数。', '解除格擋後，再次開始格擋前隨機等待的遊戲刻數。')
text('AutoBlocking.PauseOnUnblock', 'Random ticks to pause attacks after releasing a blocking item.', '解除格挡物品使用后，随机暂停攻击的游戏刻数。', '解除格擋物品使用後，隨機暫停攻擊的遊戲刻數。')
text('AutoBlocking.Chance', 'Chance to start a new block action.', '开始一次新格挡操作的概率。', '開始一次新格擋操作的機率。')
text('AutoBlocking.Blink', 'Hold movement packets while blocking for up to this many ticks.', '格挡时暂存移动数据包，最多持续此游戏刻数。', '格擋時暫存移動封包，最多持續此遊戲刻數。')
text('AutoBlocking.PrioritizeBlocking', 'Delay attacks while blocking has priority.', '格挡处于优先状态时延后攻击。', '格擋處於優先狀態時延後攻擊。')
text('AutoBlocking.OnScanRange', 'Allow blocking while a target is within scan range but outside attack range.', '目标处于扫描距离内、攻击距离外时也允许格挡。', '目標處於掃描距離內、攻擊距離外時也允許格擋。')
text('AutoBlocking.OnlyWhenInDanger', 'Block only when the target threatens the player or is within the forced range.', '只在目标能够威胁玩家，或处于强制格挡距离内时格挡。', '只在目標能夠威脅玩家，或處於強制格擋距離內時格擋。')
text('AutoBlocking.OnlyWhenInDanger.Enabled', 'Enable danger-based blocking.', '启用基于受击危险的格挡判断。', '啟用基於受擊危險的格擋判斷。')
text('AutoBlocking.OnlyWhenInDanger.Tolerance', 'Expand the target attack check by this distance.', '为对手攻击检查追加此距离容差。', '為對手攻擊檢查追加此距離容差。')
text('AutoBlocking.OnlyWhenInDanger.ForceActiveRange', 'Always treat a target inside this configured distance interval as dangerous.', '目标距离落入此区间时，始终视为存在受击危险。', '目標距離落入此區間時，始終視為存在受擊危險。')
text('AutoBlocking.AssumeShield', 'Compatibility option for older item protocols without a blocks-attacks component.', '兼容不含格挡攻击组件的旧物品协议。', '相容不含格擋攻擊元件的舊物品協定。')
text('FailSwing', 'Swing when a target is nearby but the real attack ray does not hit it.', '目标位于附近、实际攻击射线未命中时模拟挥手。', '目標位於附近、實際攻擊射線未命中時模擬揮手。')
text('FailSwing.Enabled', 'Enable missed-target swings.', '启用未命中目标时的挥手。', '啟用未命中目標時的揮手。')
text('FailSwing.AdditionalRange', 'Random extra distance within which missed-target swings may occur.', '允许出现未命中挥手的随机额外距离。', '允許出現未命中揮手的隨機額外距離。')
text('FailSwing.NotifyWhenFail', 'Show no feedback, a small fading box, or a sound for missed swings.', '选择未命中挥手时不反馈、显示渐隐小方框，或播放声音。', '選擇未命中揮手時不回饋、顯示漸隱小方框，或播放音效。')
text('FailSwing.NotifyWhenFail.Box', 'Mark missed hit points with fading boxes.', '在未命中的位置绘制渐隐小方框。', '在未命中的位置繪製漸隱小方框。')
text('FailSwing.NotifyWhenFail.Box.Fade', 'Lifetime of missed-hit boxes in seconds.', '未命中标记方框的持续秒数。', '未命中標記方框的持續秒數。')
text('FailSwing.NotifyWhenFail.Box.Color', 'ARGB color of missed-hit boxes.', '未命中标记方框的 ARGB 颜色。', '未命中標記方框的 ARGB 色彩。')
text('FailSwing.NotifyWhenFail.Box.Rainbow', 'Animate missed-hit box colors through the rainbow.', '让未命中标记方框的颜色循环变化。', '讓未命中標記方框的色彩循環變化。')
text('FailSwing.NotifyWhenFail.Sound', 'Play the source missed-hit sound.', '未命中时播放原版提示音。', '未命中時播放原版提示音效。')
text('FailSwing.NotifyWhenFail.Sound.Volume', 'Missed-hit sound volume percentage.', '未命中提示音的音量百分比。', '未命中提示音效的音量百分比。')
text('FailSwing.NotifyWhenFail.Sound.Pitch', 'Pitch of the missed-hit sound.', '未命中提示音的音高。', '未命中提示音效的音高。')
text('FightBot', 'Navigate independently towards combat positions or a named leader.', '独立寻路前往战斗位置，或跟随指定名称的领队。', '獨立尋路前往戰鬥位置，或跟隨指定名稱的領隊。')
text('FightBot.Enabled', 'Enable combat movement and its independent navigator.', '启用战斗移动及其独立寻路器。', '啟用戰鬥移動及其獨立尋路器。')
text('FightBot.Render', 'Draw the independently calculated combat route and navigation actions.', '绘制独立计算的战斗路径与寻路动作。', '繪製獨立計算的戰鬥路徑與尋路動作。')
text('FightBot.OpponentRange', 'Distance used to decide whether the opponent is close enough to threaten the player.', '用于判断对手是否已经靠近到能够威胁玩家的距离。', '用於判斷對手是否已經靠近到能夠威脅玩家的距離。')
text('FightBot.DangerousYaw', 'Avoid positions inside this angular sector around the opponent facing direction.', '避开对手朝向周围此角度范围内的位置。', '避開對手朝向周圍此角度範圍內的位置。')
text('FightBot.RunawayOnCooldown', 'Move away when an opponent is close and the clicker is not ready to attack.', '对手靠近且点击器尚未准备攻击时，向远离对手的方向移动。', '對手靠近且點擊器尚未準備攻擊時，向遠離對手的方向移動。')
text('FightBot.TargetFilter', 'Additional target selection rules while FightBot is running.', '自动战斗寻路运行时附加的目标筛选条件。', '自動戰鬥尋路執行時附加的目標篩選條件。')
text('FightBot.TargetFilter.Range', 'Maximum boxed distance for FightBot target selection.', '自动战斗寻路选择目标时允许的最大碰撞箱距离。', '自動戰鬥尋路選擇目標時允許的最大碰撞箱距離。')
text('FightBot.TargetFilter.VisibleOnly', 'Select only entities with a real line of sight from the player.', '只选择玩家实际能够看到的实体。', '只選擇玩家實際能夠看到的實體。')
text('FightBot.TargetFilter.NotWhenVoid', 'Require a real block collision below the target box down to Y=-64.', '要求目标碰撞箱下方直到 Y=-64 的范围内存在真实方块碰撞。', '要求目標碰撞箱下方直到 Y=-64 的範圍內存在真實方塊碰撞。')
text('FightBot.Leader', 'Give a matching named player priority over combat movement goals.', '存在名称匹配的玩家时，优先跟随领队目标。', '存在名稱相符的玩家時，優先跟隨領隊目標。')
text('FightBot.Leader.Enabled', 'Enable named-leader following.', '启用按名称跟随领队。', '啟用按名稱跟隨領隊。')
text('FightBot.Leader.Username', 'Exact profile name of the player to follow.', '要跟随玩家的准确档案名称。', '要跟隨玩家的確切個人檔案名稱。')
text('FightBot.Leader.Radius', 'Desired distance from the leader and threshold for the movement jump assist.', '期望与领队保持的距离，同时用于判断移动跳跃辅助。', '期望與領隊保持的距離，同時用於判斷移動跳躍輔助。')
text('RangeIndicator', 'Draw the active attack ranges around the player.', '在玩家周围绘制当前攻击距离。', '在玩家周圍繪製目前攻擊距離。')
text('RangeIndicator.Enabled', 'Enable attack-range rendering.', '启用攻击距离渲染。', '啟用攻擊距離繪製。')
text('RangeIndicator.ColorMode', 'Choose static colors, a rainbow, or colors interpolated by target distance.', '选择固定颜色、彩虹色，或按目标距离插值着色。', '選擇固定色彩、彩虹色，或按目標距離插值著色。')
text('RangeIndicator.IdleColor', 'Color while no attack target is selected.', '没有选中攻击目标时使用的颜色。', '沒有選中攻擊目標時使用的色彩。')
text('RangeIndicator.ActiveColor', 'Color while an attack target is selected.', '选中攻击目标时使用的颜色。', '選中攻擊目標時使用的色彩。')
text('RangeIndicator.Outline', 'Draw an outline around the range surface.', '在距离表面周围绘制轮廓线。', '在距離表面周圍繪製輪廓線。')
text('RangeIndicator.OutlineColor', 'ARGB color of the range outline.', '距离轮廓线的 ARGB 颜色。', '距離輪廓線的 ARGB 色彩。')
text('RangeIndicator.PulseAnimation', 'Animate the range radius with a pulse.', '以脉冲方式改变距离圆环的半径。', '以脈衝方式改變距離圓環的半徑。')
text('RangeIndicator.PulseSpeed', 'Speed of the radius pulse animation.', '半径脉冲动画的速度。', '半徑脈衝動畫的速度。')
text('RangeIndicator.PulseIntensity', 'Strength of the range radius pulse.', '距离半径脉冲变化的强度。', '距離半徑脈衝變化的強度。')
text('RangeIndicator.FadeAnimation', 'Fade the range indicator as its active state changes.', '指示器的激活状态变化时应用淡入淡出。', '指示器的啟用狀態變化時套用淡入淡出。')
text('RangeIndicator.FadeSpeed', 'Interpolation speed of the indicator fade.', '指示器淡入淡出的插值速度。', '指示器淡入淡出的插值速度。')
text('RangeIndicator.WallRangeColor', 'ARGB color of the through-walls distance ring; zero alpha hides it.', '遮挡攻击距离环的 ARGB 颜色；透明度为零时隐藏。', '遮擋攻擊距離環的 ARGB 色彩；透明度為零時隱藏。')
text('RangeIndicator.ScanRangeColor', 'ARGB color of the target scan distance ring; zero alpha hides it.', '目标扫描距离环的 ARGB 颜色；透明度为零时隐藏。', '目標掃描距離環的 ARGB 色彩；透明度為零時隱藏。')
text('RangeIndicator.OpponentRangeColor', 'ARGB color of the three-block opponent reference ring; zero alpha hides it.', '三格半径的对手参考距离环的 ARGB 颜色；透明度为零时隐藏。', '三格半徑的對手參考距離環的 ARGB 色彩；透明度為零時隱藏。')
text('RangeIndicator.HideWhenDead', 'Hide the indicator when the player is dead.', '玩家死亡时隐藏距离指示。', '玩家死亡時隱藏距離指示。')
text('RangeIndicator.HideWhenSpectator', 'Hide the indicator in spectator mode.', '旁观模式下隐藏距离指示。', '旁觀模式下隱藏距離指示。')
text('RangeIndicator.HideInVehicle', 'Hide the indicator while the player is a passenger.', '玩家乘坐载具时隐藏距离指示。', '玩家乘坐載具時隱藏距離指示。')
text('RangeIndicator.RespectInventorySetting', 'Apply KillAura inventory restrictions to indicator visibility.', '距离指示的显示遵守杀戮光环的物品栏限制。', '距離指示的顯示遵守殺戮光環的物品欄限制。')
text('RangeIndicator.CanBeCovered', 'Allow world geometry to occlude the range indicator.', '允许世界方块遮挡距离指示。', '允許世界方塊遮擋距離指示。')


def update(folder):
    rod = runpy.run_path(str(HERE / 'update-auto-rod-locales.py'))
    bow = runpy.run_path(str(HERE / 'update-auto-bow-locales.py'))
    rendering = runpy.run_path(str(HERE / 'update-target-renderer-locales.py'))['update']
    texts = {k: v for k, v in rod['TEXTS'].items()
             if k.startswith('aimPoint') or k.startswith('requires.')}
    texts.update({k.removeprefix('bowAimbot.'): v for k, v in bow['TEXTS'].items()
                  if k.startswith('bowAimbot.target')})
    texts.update(TEXTS)
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = folder / f'display_{locale}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        entries = {'': LABELS['KillAura'][index - 1]}
        entries.update({k: v[index] for k, v in texts.items()})
        put_module(data, PREFIX, 'KillAura', entries, {k: v[0] for k, v in texts.items()})
        for labels in [rod['LABELS'], bow['LABELS'], LABELS]:
            data['labels'].update({k: v[index - 1] for k, v in labels.items()})
        for field in ['descriptions', 'englishDescriptions']:
            target = data.setdefault(field, {})
            for key, value in list(target.items()):
                old = 'liquidbounce.module.projectilePuncher.rotations'
                if key == old + '.description' or key.startswith(old + '.'):
                    target[key.replace(old, PREFIX + '.rotations', 1)] = value
                old = 'liquidbounce.module.projectilePuncher.clicker.'
                if key.startswith(old):
                    target[key.replace(old, PREFIX + '.clicker.', 1)] = value
                old = 'liquidbounce.module.autoWalk.'
                if key.startswith(old):
                    suffix = key.removeprefix(old)
                    if suffix.split('.', 1)[0] in {
                        'allowBreak', 'allowPlace', 'allowParkour', 'riskyJumps', 'maxFall',
                        'pathRange', 'allowFly', 'allowElytra', 'elytraRockets', 'allowBoat',
                        'allowEat', 'render',
                    }:
                        target[PREFIX + '.fightBot.' + suffix] = value
        # Source-specific timing overrides the shared combat rotation tree.
        for key, values in TEXTS.items():
            if key.startswith('rotations.'):
                data['descriptions'][PREFIX + '.' + key] = values[index]
                if 'englishDescriptions' in data:
                    data['englishDescriptions'][PREFIX + '.' + key] = values[0]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
        rendering(locale, path, PREFIX + '.targetRendering')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder', type=Path,
                        default=HERE.parent / 'liquidbounce-interop/resources/localization')
    args = parser.parse_args()
    update(args.folder)
