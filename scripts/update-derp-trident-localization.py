#!/usr/bin/env python3
"""Owned EN/CN/TW help for the pinned Derp and TridentBoost settings."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'

HELP = {
    'derp.yaw.static.yaw': (
        'Use this fixed horizontal rotation.', '使用此固定水平朝向。', '使用此固定水平朝向。'),
    'derp.yaw.offset.offset': (
        'Add this angle to the player\'s visible horizontal rotation.', '在玩家可见的水平朝向上增加此角度。', '在玩家可見的水平朝向上增加此角度。'),
    'derp.yaw.jitter.forwardTicks': (
        'Follow the player\'s horizontal rotation for this many game ticks in each jitter cycle.', '每轮抖动中，跟随玩家水平朝向的游戏刻数。', '每輪抖動中，跟隨玩家水平朝向的遊戲刻數。'),
    'derp.yaw.jitter.backwardTicks': (
        'Face 180 degrees behind the player\'s horizontal rotation for this many game ticks in each jitter cycle.', '每轮抖动中，朝向玩家水平朝向后方 180 度的游戏刻数。', '每輪抖動中，朝向玩家水平朝向後方 180 度的遊戲刻數。'),
    'derp.yaw.spin.speed': (
        'Add this many degrees to the accumulated horizontal rotation each game tick; a negative value reverses the spin.', '每个游戏刻在累计水平朝向上增加此度数；负数反向旋转。', '每個遊戲刻在累計水平朝向上增加此度數；負數反向旋轉。'),
    'derp.pitch.static.pitch': (
        'Use this fixed vertical rotation; SafePitch limits the result to -90 through 90 degrees.', '使用此固定垂直朝向；安全俯仰会将结果限制在 -90 至 90 度。', '使用此固定垂直朝向；安全俯仰會將結果限制在 -90 至 90 度。'),
    'derp.pitch.offset.offset': (
        'Add this angle to the player\'s visible vertical rotation before applying SafePitch.', '在玩家可见的垂直朝向上增加此角度，再应用安全俯仰限制。', '在玩家可見的垂直朝向上增加此角度，再套用安全俯仰限制。'),
    'derp.safePitch': (
        'Limit the requested vertical rotation to -90 through 90 degrees; Random also chooses within that range.', '将请求的垂直朝向限制在 -90 至 90 度；随机模式也在此范围内取值。', '將請求的垂直朝向限制在 -90 至 90 度；隨機模式也在此範圍內取值。'),
    'derp.notDuringSprint': (
        'Skip Derp rotation requests while the sprint key is held or the player is sprinting.', '按住疾跑键或玩家正在疾跑时，暂停发起搞怪转向。', '按住疾跑鍵或玩家正在疾跑時，暫停發起搞怪轉向。'),
    'tridentBoost.horizontalMultiplier': (
        'Multiply the horizontal components of the trident\'s Riptide dash by this factor.', '将三叉戟激流冲刺的水平分量乘以此倍率。', '將三叉戟激流衝刺的水平分量乘以此倍率。'),
    'tridentBoost.verticalMultiplier': (
        'Multiply the vertical component of the trident\'s Riptide dash by this factor.', '将三叉戟激流冲刺的垂直分量乘以此倍率。', '將三叉戟激流衝刺的垂直分量乘以此倍率。'),
    'tridentBoost.onLand': (
        'Allow Riptide use without being in water or rain.', '不在水中或雨中时仍允许使用激流。', '不在水中或雨中時仍允許使用激流。'),
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
