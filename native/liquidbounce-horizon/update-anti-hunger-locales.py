#!/usr/bin/env python3
"""Merge AntiHunger settings help and verify the pinned original notifications."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'native/liquidbounce-interop/resources/localization'
BASE = 'liquidbounce.module.antiHunger.'
ENGLISH = {
    'noSprint.description': 'Prevents sprinting through movement input and outgoing sprint state. Swimming is excluded unless WhileSwimming is enabled.',
    'noSprint.enabled.description': 'Prevent sprinting through movement input and outgoing sprint state. Enabled by default.',
    'noSprint.whileSwimming.description': 'Also prevent sprinting while swimming. Disabled by default.',
    'keepFloating.description': 'Send movement as airborne while not falling. Vehicles, block breaking and contact with water are excluded. Enabled by default.',
}
SIMPLIFIED = {
    'noSprint.description': '通过移动输入和发送的疾跑状态阻止疾跑。除非启用“游泳时”，否则游泳时不生效。',
    'noSprint.enabled.description': '通过移动输入和发送的疾跑状态阻止疾跑。默认开启。',
    'noSprint.whileSwimming.description': '游泳时也阻止疾跑。默认关闭。',
    'keepFloating.description': '未下落时将移动状态发送为离地。乘坐载具、破坏方块或接触水时不生效。默认开启。',
}
TRADITIONAL = {
    'noSprint.description': '透過移動輸入和傳送的疾跑狀態阻止疾跑。除非啟用「游泳時」，否則游泳時不生效。',
    'noSprint.enabled.description': '透過移動輸入和傳送的疾跑狀態阻止疾跑。預設開啟。',
    'noSprint.whileSwimming.description': '游泳時也阻止疾跑。預設關閉。',
    'keepFloating.description': '未下落時將移動狀態傳送為離地。乘坐載具、破壞方塊或接觸水時不生效。預設開啟。',
}


def main():
    output = []
    for language, descriptions in [('zh_cn', SIMPLIFIED), ('zh_tw', TRADITIONAL)]:
        path = RESOURCES / f'display_{language}.json'
        data = json.loads(path.read_text(encoding='utf-8'))
        data.setdefault('descriptions', {}).update({BASE + k: v for k, v in descriptions.items()})
        if language == 'zh_cn':
            data.setdefault('englishDescriptions', {}).update({BASE + k: v for k, v in ENGLISH.items()})
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        output.append({'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    for language in ['en_us', 'zh_cn', 'zh_tw']:
        path = RESOURCES / f'{language}.json'
        original = ROOT / f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json'
        local = json.loads(path.read_text(encoding='utf-8'))
        source = json.loads(original.read_text(encoding='utf-8'))
        for message in ['warnSprint', 'warnBreak']:
            key = BASE + 'messages.' + message
            if local[key] != source[key]:
                raise ValueError(f'Original notification changed: {language} {key}')
    print(json.dumps({'updated': output, 'originalNotificationsVerified': 6}))


if __name__ == '__main__':
    main()
