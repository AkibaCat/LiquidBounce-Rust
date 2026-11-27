#!/usr/bin/env python3
"""Merge owned Damage, AutoDisable and Plugins descriptions and choice labels."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[2]
HELP = {
    'en_us': {
        'damage.mode': 'Select the NCP, AAC or Verus self-damage method.',
        'damage.damage': 'Requested damage in health points. NCP scales the movement packet repetitions and AAC scales upward velocity; Verus uses its fixed movement sequence.',
        'autoDisable.modules': 'Configured module names. The original keeps this setting separate from active membership; use .autodisable add, remove, clear or list to manage the active selection.',
        'autoDisable.on': 'Disable selected modules after a server position correction, local player death, world replacement or disconnect. At least one trigger remains selected.',
        'plugins.try': 'Try /plugins, /bukkit:plugins, and command suggestions in the selected order. The module turns off when detection completes.',
    },
    'zh_cn': {
        'damage.mode': '选择 NCP、AAC 或 Verus 自伤方式。',
        'damage.damage': '请求的伤害量，单位为生命值。NCP 按此值增加移动包次数，AAC 按此值增加向上速度；Verus 使用固定移动序列。',
        'autoDisable.modules': '配置中的模块名称。原版会将此设置与实际生效的成员列表分别保存；使用 .autodisable add、remove、clear 或 list 管理实际列表。',
        'autoDisable.on': '服务器纠正位置、本地玩家死亡、世界切换或断开连接时关闭选定模块。至少保留一个触发条件。',
        'plugins.try': '按所选顺序尝试 /plugins、/bukkit:plugins 和命令补全，检测完成后自动关闭。',
    },
    'zh_tw': {
        'damage.mode': '選擇 NCP、AAC 或 Verus 自傷方式。',
        'damage.damage': '請求的傷害量，單位為生命值。NCP 按此值增加移動封包次數，AAC 按此值增加向上速度；Verus 使用固定移動序列。',
        'autoDisable.modules': '設定中的模組名稱。原版會將此設定與實際生效的成員清單分別儲存；使用 .autodisable add、remove、clear 或 list 管理實際清單。',
        'autoDisable.on': '伺服器修正位置、本機玩家死亡、世界切換或中斷連線時關閉選定模組。至少保留一個觸發條件。',
        'plugins.try': '按所選順序嘗試 /plugins、/bukkit:plugins 和命令補全，偵測完成後自動關閉。',
    },
}
LABELS = {
    'zh_cn': {'Flag': '服务器回退', 'Death': '玩家死亡', 'WorldChange': '世界切换', 'Disconnect': '断开连接', 'Slash': '/plugins', 'ColonSlash': '/bukkit:plugins', 'TabComplete': '命令补全'},
    'zh_tw': {'Flag': '伺服器回退', 'Death': '玩家死亡', 'WorldChange': '世界切換', 'Disconnect': '中斷連線', 'Slash': '/plugins', 'ColonSlash': '/bukkit:plugins', 'TabComplete': '命令補全'},
}
MODULES = ('damage', 'autoDisable', 'plugins')
for language in ('zh_cn', 'zh_tw'):
    target = ROOT / f'native/liquidbounce-interop/resources/localization/display_{language}.json'
    data = json.loads(target.read_text(encoding='utf8'))
    upstream = json.loads((ROOT / f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
    for module in MODULES:
        key = f'liquidbounce.module.{module}.description'
        data['descriptions'][key] = upstream[key]
    data['descriptions'].update({f'liquidbounce.module.{key}.description': value for key, value in HELP[language].items()})
    for key, value in LABELS[language].items():
        data['labels'].setdefault(key, value)
    if language == 'zh_cn':
        english = json.loads((ROOT / 'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
        for module in MODULES:
            key = f'liquidbounce.module.{module}.description'
            data['englishDescriptions'][key] = english[key]
        data['englishDescriptions'].update({f'liquidbounce.module.{key}.description': value for key, value in HELP['en_us'].items()})
    target.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
print('Merged Damage/AutoDisable/Plugins simplified and traditional Chinese help')
