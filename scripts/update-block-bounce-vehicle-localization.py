#!/usr/bin/env python3
"""Owned EN/CN/TW help for complete BlockBounce and VehicleBoost leaves."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
HELP = {
    'blockBounce.motion': (
        'Add this amount to normal jump power when jumping on a slime block, honey block or bed.',
        '在黏液块、蜂蜜块或床上起跳时，在正常跳跃力上增加此数值。',
        '在史萊姆方塊、蜂蜜塊或床上起跳時，在正常跳躍力上增加此數值。'),
    'vehicleBoost.horizontalSpeed': (
        'When leaving a vehicle, replace horizontal velocity with this speed in the direction the player is facing.',
        '离开载具时，按玩家朝向将水平速度替换为此数值。',
        '離開載具時，按玩家朝向將水平速度替換為此數值。'),
    'vehicleBoost.verticalSpeed': (
        'When leaving a vehicle, replace vertical velocity with this upward speed.',
        '离开载具时，将垂直速度替换为此向上速度。',
        '離開載具時，將垂直速度替換為此向上速度。'),
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
