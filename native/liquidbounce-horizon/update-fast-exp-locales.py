#!/usr/bin/env python3
"""FastExp labels and help in the actual nested display dictionary."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[2]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
LABELS = {
    'NoWaste': ('避免浪费', '避免浪費'),
    'MinDurabilityToStartRepair': ('开始修复的剩余耐久', '開始修復的剩餘耐久'),
    'MaxDurabilityToContinueRepair': ('继续修复的耐久百分比', '繼續修復的耐久百分比'),
    'ThrowMode': ('投掷模式', '投擲模式'),
    'TicksPerItem': ('每瓶间隔刻数', '每瓶間隔刻數'),
    'ItemsPerTick': ('每刻投掷瓶数', '每刻投擲瓶數'),
    'CombatPauseTime': ('暂停战斗时间', '暫停戰鬥時間'),
    'SlotResetDelay': ('恢复快捷栏延迟', '恢復快捷列延遲'),
}
TEXTS = {
    '': ('Throw experience bottles to repair Mending equipment.', '自动投掷经验瓶，修复带有经验修补的装备。', '自動投擲經驗瓶，修復帶有經驗修補的裝備。'),
    'rotate.enabled': ('Aim down and wait for the server rotation before throwing.', '投掷前向下转向，并等待服务器朝向到位。', '投擲前向下轉向，並等待伺服器朝向到位。'),
    'noWaste.enabled': ('Estimate bottles required by armor and the other hand, then wait for nearby experience orbs.', '根据护甲及另一只手物品的损耗估算所需经验瓶，并等待经验球被吸收。', '依護甲及另一隻手物品的損耗估算所需經驗瓶，並等待經驗球被吸收。'),
    'noWaste.minDurabilityToStartRepair': ('Begin a repair run when an eligible item has at most this much durability left.', '可修复物品的剩余耐久不高于此值时开始修复。', '可修復物品的剩餘耐久不高於此值時開始修復。'),
    'noWaste.maxDurabilityToContinueRepair': ('Continue a repair run while an eligible item remains at or below this durability percentage.', '一轮投掷后，仍有可修复物品的耐久百分比不高于此值时继续修复。', '一輪投擲後，仍有可修復物品的耐久百分比不高於此值時繼續修復。'),
    'throwMode': ('Choose a randomized interval or a randomized number of bottles per tick.', '选择随机投掷间隔，或每刻随机投掷多瓶。', '選擇隨機投擲間隔，或每刻隨機投擲多瓶。'),
    'throwMode.normal.ticksPerItem': ('Randomized number of ticks per bottle; fractional rates accumulate between ticks.', '每瓶使用的随机刻数；不足一瓶的投掷量会累积到后续游戏刻。', '每瓶使用的隨機刻數；不足一瓶的投擲量會累積到後續遊戲刻。'),
    'throwMode.fast.itemsPerTick': ('Randomized number of bottles per tick; fractional counts accumulate.', '每刻投掷的随机瓶数；小数部分会继续累积。', '每刻投擲的隨機瓶數；小數部分會繼續累積。'),
    'combatPauseTime': ('Pause automatic combat for at least this many ticks when starting a throw action.', '开始投掷动作时，至少暂停自动战斗这么多游戏刻。', '開始投擲動作時，至少暫停自動戰鬥這麼多遊戲刻。'),
    'slotResetDelay': ('Randomized delay before releasing this action\'s silent hotbar selection.', '本次动作静默选择快捷栏后，随机等待这么多刻再恢复。', '本次動作靜默選擇快捷列後，隨機等待這麼多刻再恢復。'),
}

def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        data['moduleNames']['FastExp'] = ('快速经验修补', '快速經驗修補')[index - 1]
        data['labels'].update({k: v[index - 1] for k, v in LABELS.items()})
        for field in ['descriptions', 'englishDescriptions']:
            for key, value in list(data.get(field, {}).items()):
                old = 'liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
                if key.startswith(old):
                    data[field][key.replace(old, 'liquidbounce.module.fastExp.rotate.rotations')] = value
        for suffix, value in TEXTS.items():
            key = 'liquidbounce.module.fastExp' + ('.' + suffix if suffix else '') + '.description'
            data['descriptions'][key] = value[index]
            if locale == 'zh_cn':
                data['englishDescriptions'][key] = value[0]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
        print(locale, len(TEXTS), 'FastExp help entries')

if __name__ == '__main__':
    main()
