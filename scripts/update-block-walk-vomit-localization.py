#!/usr/bin/env python3
"""Owned full BlockWalk and Vomit InventoryConstraints display help."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
HELP = {
    'blockWalk.blocks': (
        'Treat selected blocks as full cubes during collision scans. The defaults are cobweb and snow; visual and directly queried block shapes retain their normal geometry.',
        '碰撞扫描时，将选中的方块视为完整立方体，默认选择蛛网和雪。方块外观与直接查询的方块形状保持原状。',
        '碰撞掃描時，將選取的方塊視為完整立方體，預設選取蜘蛛網和雪。方塊外觀與直接查詢的方塊形狀維持原狀。'),
    'ghostHand.targetedBlocks': (
        'Prioritize ray hits on selected blocks through other blocks. If no selected block is hit, use the ordinary block and fluid raycast. Defaults include chests and all shulker boxes.',
        '射线优先命中所选方块，可穿过其他方块与其交互；若没有命中所选方块，则使用普通方块与流体射线结果。默认包括各种箱子与全部潜影盒。',
        '射線優先命中所選方塊，可穿過其他方塊與其互動；若沒有命中所選方塊，則使用一般方塊與流體射線結果。預設包含各種儲物箱與全部界伏盒。'),
    'vomit.constraints': (
        'Timing and requirements for the module\'s inventory action plans.',
        '设置此模块物品栏操作计划的时序与执行条件。',
        '設定此模組物品欄操作計畫的時序與執行條件。'),
    'vomit.constraints.startDelay': (
        'Random delay in milliseconds when opening the inventory or starting after a recent open. Cursor travel can add a distance-based delay.',
        '打开物品栏，或在刚打开物品栏后开始操作时，随机等待的毫秒数。光标移动距离还会增加额外延迟。',
        '開啟物品欄，或在剛開啟物品欄後開始操作時，隨機等待的毫秒數。游標移動距離還會增加額外延遲。'),
    'vomit.constraints.clickDelay': (
        'Random delay in milliseconds between inventory actions. In creative mode, a maximum of zero enables filling and throwing all available empty storage slots in one plan.',
        '物品栏操作之间随机等待的毫秒数。创造模式下，若范围上限不大于 0，一次计划会填入并丢出所有可用空存储槽位中的物品。',
        '物品欄操作之間隨機等待的毫秒數。創造模式下，若範圍上限不大於 0，一次計畫會填入並丟出所有可用空儲存欄位中的物品。'),
    'vomit.constraints.closeDelay': (
        'Random delay in milliseconds before closing the inventory or a container screen when the action plan needs to close it.',
        '操作计划需要关闭物品栏或容器界面时，在关闭前随机等待的毫秒数。',
        '操作計畫需要關閉物品欄或容器畫面時，在關閉前隨機等待的毫秒數。'),
    'vomit.constraints.missChance': (
        'Random chance to simulate a miss on the first ordinary click of a chain. Throw actions do not simulate misses.',
        '操作链首次普通点击时，随机模拟误点的概率。丢弃操作不会模拟误点。',
        '操作鏈首次普通點擊時，隨機模擬誤點的機率。丟棄操作不會模擬誤點。'),
    'vomit.constraints.slotMultiplier': (
        'Extra cursor-travel delay per slot of distance, multiplied by a random base delay. Start travel begins at the screen center; later travel begins at the previous slot.',
        '按跨越槽位距离增加光标移动延迟，额外时间为随机基础延迟乘以距离及此系数。首次从屏幕中心计算，之后从上次点击的槽位计算。',
        '依跨越欄位距離增加游標移動延遲，額外時間為隨機基礎延遲乘以距離及此係數。首次從螢幕中央計算，之後從上次點擊的欄位計算。'),
    'vomit.constraints.requires': (
        'Require selected conditions before actions: no movement or jumping, no rotation change, no item use, no mining, no combat, or an open player inventory when the action needs it. All selected conditions must pass.',
        '操作前检查所选条件：不移动或跳跃、不改变转向、不使用物品、不挖掘、不处于战斗，或在操作需要时已打开玩家物品栏。所有已选条件均须满足。',
        '操作前檢查所選條件：不移動或跳躍、不改變轉向、不使用物品、不挖掘、不處於戰鬥，或在操作需要時已開啟玩家物品欄。所有已選條件均須滿足。'),
    'vomit.constraints.millisecondDelays': (
        'Saved compatibility marker for millisecond delays. Legacy files lacking both this marker and an Interop value type convert delay endpoints from ticks by multiplying by 50.',
        '用于保存毫秒延迟格式的兼容标记。旧配置若同时缺少此标记和 Interop 值类型，会将延迟范围端点乘以 50，从游戏刻换算为毫秒。',
        '用於儲存毫秒延遲格式的相容標記。舊設定若同時缺少此標記和 Interop 值類型，會將延遲範圍端點乘以 50，從遊戲刻換算為毫秒。'),
}


def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        display = json.loads(path.read_text(encoding='utf-8'))
        for key, texts in HELP.items():
            key = 'liquidbounce.module.' + key + '.description'
            display['descriptions'][key] = texts[index]
            if locale == 'zh_cn':
                display['englishDescriptions'][key] = texts[0]
        path.write_text(json.dumps(display, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        print(f'{locale}: {len(display["descriptions"])} owned descriptions')


if __name__ == '__main__':
    main()
