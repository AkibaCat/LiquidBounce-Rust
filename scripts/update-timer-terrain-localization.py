#!/usr/bin/env python3
"""Owned Timer and TerrainSpeed display help; wire identifiers stay unchanged."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
TEXTS={
'timer.mode':('Classic uses a constant speed; Pulse alternates normal and boosted segments; Boost manages speed against the shared timer balance.','Classic 使用固定速度；Pulse 交替执行常速与加速时段；Boost 根据共享计时余额控制速度。','Classic 使用固定速度；Pulse 交替執行常速與加速時段；Boost 根據共享計時餘額控制速度。'),
'timer.mode.classic.speed':('Request this game speed every game tick. Default 2; the original slider range is 0.1–20.','每个游戏刻请求此游戏速度。默认 2；原始滑块范围为 0.1–20。','每個遊戲刻請求此遊戲速度。預設 2；原始滑桿範圍為 0.1–20。'),
'timer.mode.pulse.normalSpeed':('Speed during the normal segment. Default 0.5; slider 0.1–20.','常速时段的游戏速度。默认 0.5；滑块范围 0.1–20。','常速時段的遊戲速度。預設 0.5；滑桿範圍 0.1–20。'),
'timer.mode.pulse.normalSpeedTicks':('Game ticks in each normal segment. Default 20; slider 1–500.','每段常速持续的游戏刻数。默认 20；滑块范围 1–500。','每段常速持續的遊戲刻數。預設 20；滑桿範圍 1–500。'),
'timer.mode.pulse.boostSpeed':('Speed during the boosted segment. Default 2; slider 0.1–20.','加速时段的游戏速度。默认 2；滑块范围 0.1–20。','加速時段的遊戲速度。預設 2；滑桿範圍 0.1–20。'),
'timer.mode.pulse.boostSpeedTicks':('Game ticks in each boosted segment. Default 20; slider 1–500.','每段加速持续的游戏刻数。默认 20；滑块范围 1–500。','每段加速持續的遊戲刻數。預設 20；滑桿範圍 1–500。'),
'timer.mode.pulse.onMove':('Start a new segment only while the player is moving. A segment already requested keeps its lifetime when movement stops.','仅在玩家移动时开始新时段。停止移动不会提前终止已请求的时段。','僅在玩家移動時開始新時段。停止移動不會提前終止已請求的時段。'),
'timer.mode.boost.boostSpeed':('Speed requested while moving when balance is above its lower bound. Default 1.3; slider 0.1–20.','移动且余额高于下限时请求的速度。默认 1.3；滑块范围 0.1–20。','移動且餘額高於下限時請求的速度。預設 1.3；滑桿範圍 0.1–20。'),
'timer.mode.boost.slowSpeed':('Speed requested while stationary to rebuild balance, except in inventory and container screens. Default 0.6; slider 0.1–20.','静止时请求此速度来恢复余额，背包与容器界面除外。默认 0.6；滑块范围 0.1–20。','靜止時請求此速度來恢復餘額，背包與容器介面除外。預設 0.6；滑桿範圍 0.1–20。'),
'timer.mode.boost.normalizeDuringCombat':('Request normal speed while the shared combat manager reports combat. Enabled by default.','共享战斗管理器判定处于战斗时请求正常速度。默认开启。','共享戰鬥管理器判定處於戰鬥時請求正常速度。預設開啟。'),
'timer.mode.boost.balance':('Clamp shared balance to this range before spending or saving it. The lower bound limits running ahead; the upper bound limits saved ticks. Default 0–20 ticks; slider -100–200.','使用前将共享余额限制在此范围。下限限制可超前的刻数，上限限制可积攒的刻数。默认 0–20 刻；滑块范围 -100–200。','使用前將共享餘額限制在此範圍。下限限制可超前的刻數，上限限制可積攢的刻數。預設 0–20 刻；滑桿範圍 -100–200。'),
'timer.mode.boost.resetBalanceOnFlag':('While Boost is running, clear the balance on the render executor after receiving a player position correction, including a packet cancelled by another listener.','Boost 运行时，收到玩家位置纠正后在渲染线程清空余额，其他监听器取消的数据包也会触发。','Boost 執行時，收到玩家位置修正後在繪製執行緒清空餘額，其他監聽器取消的封包也會觸發。'),
'terrainSpeed.fastClimb.enabled':('Enable the selected fast climbing behavior.','启用所选的快速攀爬行为。','啟用所選的快速攀爬行為。'),
'terrainSpeed.fastClimb.mode':('Choose the original fast climbing mode.','选择原版快速攀爬模式。','選擇原版快速攀爬模式。'),
'terrainSpeed.fastClimb.mode.motion.motion':('Vertical motion used by Motion climbing mode.','Motion 攀爬模式使用的垂直速度。','Motion 攀爬模式使用的垂直速度。'),
'terrainSpeed.iceSpeed.enabled':('Enable ice movement changes.','启用冰面移动修改。','啟用冰面移動修改。'),
'terrainSpeed.iceSpeed.slipperiness':('Slipperiness returned for the ice blocks handled by this module.','此模块处理的冰方块所返回的滑度。','此模組處理的冰方塊所回傳的滑度。'),
'terrainSpeed.iceSpeed.motion.enabled':('Enable the additional ice motion behavior.','启用额外的冰面速度修改。','啟用額外的冰面速度修改。'),
'terrainSpeed.iceSpeed.motion.motion':('Motion value used by the additional ice movement behavior.','额外冰面移动行为使用的速度值。','額外冰面移動行為使用的速度值。'),
'terrainSpeed.waterSpeed.enabled':('Enable the water movement behavior.','启用水中移动行为。','啟用水中移動行為。'),
'terrainSpeed.waterSpeed.autoSwim':('Apply the original automatic swimming behavior in water.','在水中应用原版自动游泳行为。','在水中套用原版自動游泳行為。'),
'terrainSpeed.waterSpeed.baseSpeed.horizontal':('Horizontal speed for the base water movement behavior.','水中基础移动行为的水平速度。','水中基礎移動行為的水平速度。'),
'terrainSpeed.waterSpeed.baseSpeed.vertical':('Vertical speed for the base water movement behavior.','水中基础移动行为的垂直速度。','水中基礎移動行為的垂直速度。'),
'terrainSpeed.waterSpeed.sprintSpeed.enabled':('Use the configured water sprint speeds when sprinting.','疾跑时使用设置的水中疾跑速度。','疾跑時使用設定的水中疾跑速度。'),
'terrainSpeed.waterSpeed.sprintSpeed.horizontal':('Horizontal speed while sprinting in water.','水中疾跑时的水平速度。','水中疾跑時的水平速度。'),
'terrainSpeed.waterSpeed.sprintSpeed.vertical':('Vertical speed while sprinting in water.','水中疾跑时的垂直速度。','水中疾跑時的垂直速度。'),
}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
    path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf-8'))
    for name,text in TEXTS.items():
        key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=text[index]
        if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(locale,len(data['descriptions']))
