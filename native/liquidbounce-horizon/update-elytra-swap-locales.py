#!/usr/bin/env python3
"""Settings help for ElytraSwap's original PlayerInventoryConstraints."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'native/liquidbounce-interop/resources/localization'
BASE = 'liquidbounce.module.elytraSwap.constraints.'


def main():
    for code in ['zh_cn', 'zh_tw']:
        path = RESOURCES / f'display_{code}.json'
        data = json.loads(path.read_text(encoding='utf-8'))
        for field in ['descriptions', 'englishDescriptions'] if code == 'zh_cn' else ['descriptions']:
            values = data.setdefault(field, {})
            values.update({k.replace('liquidbounce.module.vomit.constraints.', BASE): v
                           for k, v in list(values.items())
                           if k.startswith('liquidbounce.module.vomit.constraints.')})
        if code == 'zh_cn':
            data['descriptions'][BASE + 'clickDelay.description'] = '交换时连续两次拾取操作之间随机等待的毫秒数。整条交换动作完成后才会自动关闭模块。'
            data['descriptions'][BASE + 'missChance.description'] = '操作链首次普通点击时，随机模拟误点的概率。交换链中的其余拾取操作会连续执行。'
            data['englishDescriptions'][BASE + 'clickDelay.description'] = 'Random delay in milliseconds between pickup actions. The module disables after the complete swap chain finishes.'
            data['englishDescriptions'][BASE + 'missChance.description'] = 'Chance to simulate a miss on the first normal click in an action chain. The remaining pickup actions execute in order.'
        else:
            data['descriptions'][BASE + 'clickDelay.description'] = '交換時連續兩次拾取操作之間隨機等待的毫秒數。整條交換動作完成後才會自動關閉模組。'
            data['descriptions'][BASE + 'missChance.description'] = '操作鏈首次普通點擊時，隨機模擬誤點的機率。交換鏈中的其餘拾取操作會連續執行。'
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'languages': ['zh_cn', 'zh_tw'], 'settingsHelpAdded': 8}))


if __name__ == '__main__':
    main()
