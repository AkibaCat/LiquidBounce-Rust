#!/usr/bin/env python3
"""Owned AutoWalk display labels and help, retaining original wire names."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
TEXTS = {
 'mode': ('Forward holds forward; Direction navigates where the camera faces; Goto walks to a destination; Follow tracks a named player.', 'Forward 持续前进；Direction 绕过障碍朝视线方向行走；Goto 前往坐标；Follow 跟随指定玩家。', 'Forward 持續前進；Direction 繞過障礙朝視線方向行走；Goto 前往座標；Follow 跟隨指定玩家。'),
 'mode.goto.destination': ('Block coordinates to reach. Picking ground moves the feet to the standable block above it.', '要前往的方块坐标。选择地面时，会将落脚点调整到其上可站立的位置。', '要前往的方塊座標。選擇地面時，會將落腳點調整到其上可站立的位置。'),
 'mode.follow.target': ('Exact profile name of the player to follow.', '要跟随玩家的完整游戏档案名称。', '要跟隨玩家的完整遊戲個人檔案名稱。'),
 'mode.follow.distance': ('Stop within this many blocks of the player. Default 3; range 1–10.', '与目标玩家保持的距离。默认 3 格；范围 1–10 格。', '與目標玩家保持的距離。預設 3 格；範圍 1–10 格。'),
 'allowBreak': ('Allow breaking blocks along the route.', '允许破坏路径上的方块。', '允許破壞路徑上的方塊。'),
 'allowPlace': ('Allow placing suitable hotbar blocks along the route.', '允许使用快捷栏中的合适方块搭路。', '允許使用快捷列中的合適方塊鋪路。'),
 'allowParkour': ('Search and follow jump routes across gaps.', '搜索并跟随跨越空隙的跳跃路线。', '搜尋並跟隨跨越空隙的跳躍路線。'),
 'riskyJumps': ('Allow jumps with less landing margin.', '允许落脚余量更小的冒险跳跃。', '允許落腳餘量更小的冒險跳躍。'),
 'maxFall': ('Largest permitted dry fall, additionally limited by player health.', '允许的最大非水中落差，还会受到玩家生命值限制。', '允許的最大非水中落差，還會受到玩家生命值限制。'),
 'pathRange': ('Maximum distance considered in each route search. Default 64 blocks.', '每次路径搜索考虑的最大距离，默认 64 格。', '每次路徑搜尋考慮的最大距離，預設 64 格。'),
 'allowFly': ('Use creative flight when player abilities permit it.', '玩家能力允许时使用创造飞行。', '玩家能力允許時使用創造飛行。'),
 'allowElytra': ('Use an equipped elytra to glide along the route.', '使用装备的鞘翅沿路径滑翔。', '使用裝備的鞘翅沿路徑滑翔。'),
 'elytraRockets': ('Use available fireworks to propel elytra flight.', '使用可用的烟花火箭推动鞘翅飞行。', '使用可用的煙火火箭推動鞘翅飛行。'),
 'allowBoat': ('Use a boat when travelling across water.', '跨越水面时允许使用船。', '跨越水面時允許使用船。'),
 'allowEat': ('Eat available food along the route when hungry.', '饥饿时允许在行进途中食用现有食物。', '飢餓時允許在行進途中食用現有食物。'),
 'render.enabled': ('Show the searched route, progress and destination.', '显示搜索路径、行进进度和目的地。', '顯示搜尋路徑、行進進度和目的地。'),
 'render.canBeCovered': ('Let world geometry hide route graphics.', '允许世界方块遮挡路径图形。', '允許世界方塊遮擋路徑圖形。'),
 'render.route': ('Color of the active route.', '当前路径的颜色。', '目前路徑的顏色。'),
 'render.search': ('Color of the route search preview.', '路径搜索预览的颜色。', '路徑搜尋預覽的顏色。'),
 'render.destination': ('Color of the destination marker.', '目的地标记的颜色。', '目的地標記的顏色。'),
 'render.break': ('Color of blocks scheduled to be broken.', '计划破坏的方块颜色。', '計畫破壞的方塊顏色。'),
 'render.place': ('Color of blocks scheduled to be placed.', '计划放置的方块颜色。', '計畫放置的方塊顏色。'),
 'render.target': ('Color of route targets and engaged ghost graphics.', '路线目标及交互中幽灵图形的颜色。', '路線目標及互動中幽靈圖形的顏色。'),
 'render.invalid': ('Color of a route invalidated by the world or server.', '被世界或服务器变化中断的路径颜色。', '被世界或伺服器變化中斷的路徑顏色。'),
 'render.ghost': ('Show a server position ghost when the navigation view supplies one.', '导航视图提供服务器位置时显示其幽灵轮廓。', '導航視圖提供伺服器位置時顯示其幽靈輪廓。'),
 'render.fadeOut': ('Milliseconds before completed or invalid routes disappear. Default 800.', '已完成或失效路径淡出的毫秒数，默认 800。', '已完成或失效路徑淡出的毫秒數，預設 800。'),
}
LABELS = {
 'Direction': ('视线方向', '視線方向'), 'Goto': ('前往坐标', '前往座標'), 'Follow': ('跟随', '跟隨'),
 'Destination': ('目的地', '目的地'), 'Target': ('目标', '目標'), 'Distance': ('距离', '距離'),
 'AllowBreak': ('允许破坏', '允許破壞'), 'AllowPlace': ('允许搭路', '允許鋪路'), 'AllowParkour': ('允许跳跃', '允許跳躍'),
 'RiskyJumps': ('冒险跳跃', '冒險跳躍'), 'MaxFall': ('最大落差', '最大落差'), 'PathRange': ('搜索范围', '搜尋範圍'),
 'AllowFly': ('允许飞行', '允許飛行'), 'AllowElytra': ('允许鞘翅', '允許鞘翅'), 'ElytraRockets': ('鞘翅火箭', '鞘翅火箭'),
 'AllowBoat': ('允许用船', '允許用船'), 'AllowEat': ('允许进食', '允許進食'), 'CanBeCovered': ('允许遮挡', '允許遮擋'),
 'Route': ('路径', '路徑'), 'Search': ('搜索', '搜尋'), 'Break': ('破坏', '破壞'), 'Place': ('放置', '放置'),
 'Invalid': ('失效', '失效'), 'Ghost': ('幽灵轮廓', '幽靈輪廓'), 'FadeOut': ('淡出时间', '淡出時間'),
}

for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
    path = FOLDER / f'display_{locale}.json'
    data = json.loads(path.read_text(encoding='utf8'))
    data['moduleNames']['AutoWalk'] = ('自动行走', '自動行走')[index-1]
    data['labels'].update({k: v[index-1] for k, v in LABELS.items()})
    for suffix, texts in TEXTS.items():
        key = f'liquidbounce.module.autoWalk.{suffix}.description'
        data['descriptions'][key] = texts[index]
        if locale == 'zh_cn':
            data['englishDescriptions'][key] = texts[0]
    # Reuse the complete shared rotation vocabulary under AutoWalk's own key.
    old = 'liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
    new = 'liquidbounce.module.autoWalk.rotations'
    for field in ('descriptions', 'englishDescriptions'):
        for key, value in list(data.get(field, {}).items()):
            if key.startswith(old):
                data[field][key.replace(old, new, 1)] = value
    data['descriptions']['liquidbounce.module.autoWalk.messages.arrived'] = ('已抵达 %s，%s，%s。', '已抵達 %s，%s，%s。')[index-1]
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print(locale, len(data['descriptions']))
