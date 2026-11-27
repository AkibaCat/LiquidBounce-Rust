#!/usr/bin/env python3
"""Owned help for all EntityControl and SkinDerp settings, with stable keys."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
HELP = {
    'entityControl.enforce': (
        'Saddled forces the mob saddle check to succeed; JumpStrength makes the local player report full mount jump charge. Each can be selected independently.',
        '选择“已装备鞍”可使生物的鞍具判断始终通过；选择“跳跃强度”可使本地玩家报告载具跳跃已蓄满力。两项可分别选择。',
        '選取「已裝備鞍」可使生物的鞍具判斷始終通過；選取「跳躍強度」可使本機玩家回報載具跳躍已蓄滿力。兩項可分別選取。'),
    'skinDerp.sync': (
        'Flip each selected skin part from its current state when enabled; otherwise randomly choose each selected part\'s visibility independently.',
        '开启后，将每个已选皮肤部件的当前显示状态翻转；关闭时，分别随机决定各已选部件是否显示。',
        '開啟後，將每個已選皮膚部件的目前顯示狀態反轉；關閉時，分別隨機決定各已選部件是否顯示。'),
    'skinDerp.delay': (
        'Wait this many game ticks before each skin-part update. Zero updates on each eligible tick without waiting.',
        '每次更新皮肤部件前等待的游戏刻数。设为 0 时，每个可执行的游戏刻直接更新。',
        '每次更新皮膚部件前等待的遊戲刻數。設為 0 時，每個可執行的遊戲刻直接更新。'),
    'skinDerp.parts': (
        'Choose the hat, jacket, left and right pants, left and right sleeves, and cape to change. Disabling restores the skin-part selection saved when enabled.',
        '选择要改变的帽子、外套、左右裤腿、左右袖子和披风。关闭模块时，会恢复开启时保存的皮肤部件选择。',
        '選取要改變的帽子、外套、左右褲管、左右袖子和披風。關閉模組時，會還原開啟時儲存的皮膚部件選擇。'),
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
