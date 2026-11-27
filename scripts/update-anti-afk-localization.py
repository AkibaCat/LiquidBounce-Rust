#!/usr/bin/env python3
"""Owned help for AntiAFK's complete tree and shared noncombat rotations."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
PREFIX = 'liquidbounce.module.antiAFK.mode.'
OLD = 'liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
ROTATIONS = PREFIX + 'custom.rotate.rotations'

HELP = {
    'en': {
        '': 'Choose Old forward movement and turning, random interactions, or custom anti-AFK actions.',
        'randomInteraction.interaction': 'Allow randomly chosen actions: jumping, swinging, changing slots, yaw, pitch or random movement.',
        'randomInteraction.delay': 'Game-tick range to wait after each random action; random movement also lasts a duration from this range.',
        'custom.rotate.enabled': 'Submit anti-AFK rotation requests at the configured interval.',
        'custom.rotate.ignoreOpenInventory': 'Allow anti-AFK rotation requests while the inventory is open.',
        'custom.rotate.delay': 'Wait this many game ticks before each custom rotation request.',
        'custom.rotate.angle': 'Add this angle to the current server yaw; pitch separately receives a random offset from -5 to 5 degrees.',
        'custom.swing.enabled': 'Swing the main hand at the configured interval.',
        'custom.swing.delay': 'When no swing is active, wait this many game ticks before starting another swing.',
        'custom.jump': 'Automatically jump while on the ground.',
        'custom.move': 'Continuously hold the forward key.',
        'custom.rotate.rotations': 'Configure smoothing, movement correction and return rules for custom anti-AFK rotation requests.',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.enabled': 'Use target-entity crosshair acceleration when a target exists. Anti-AFK requests have no target entity.',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.coefDistance': 'Distance contribution to acceleration. Anti-AFK requests use a target distance of zero.',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.yawCrosshairAccel': 'Horizontal acceleration used when the crosshair is on a target entity; anti-AFK requests without an entity do not use it.',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.pitchCrosshairAccel': 'Vertical acceleration used when the crosshair is on a target entity; anti-AFK requests without an entity do not use it.',
    },
    'zh_cn': {
        '': '选择旧版持续前进与转头、随机交互或自定义防挂机行为。',
        'randomInteraction.interaction': '允许随机选择的交互：跳跃、挥手、切换槽位、水平转向、垂直转向或随机移动。',
        'randomInteraction.delay': '每次随机交互后等待的游戏刻数范围；随机移动也持续此范围内的时间。',
        'custom.rotate.enabled': '按设定间隔发起防挂机转向。',
        'custom.rotate.ignoreOpenInventory': '打开物品栏时仍允许提交防挂机转向。',
        'custom.rotate.delay': '每次发起自定义转向前等待的游戏刻数。',
        'custom.rotate.angle': '在当前服务器水平朝向上增加此角度；垂直朝向另外加入 -5 至 5 度随机偏移。',
        'custom.swing.enabled': '按设定间隔挥动主手。',
        'custom.swing.delay': '未在挥手时，等待这些游戏刻数再开始下一次挥手。',
        'custom.jump': '站在地面时自动跳跃。',
        'custom.move': '持续按下前进键。',
        'custom.rotate.rotations': '设置自定义防挂机转向的平滑过程、移动修正及返回可见视角的规则。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.enabled': '存在目标实体时使用准星加速度；防挂机的转向请求没有目标实体。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.coefDistance': '距离对加速度的影响系数；防挂机转向采用零目标距离。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.yawCrosshairAccel': '准星指向目标实体时使用的水平加速度；无目标实体的防挂机请求不使用此值。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.pitchCrosshairAccel': '准星指向目标实体时使用的垂直加速度；无目标实体的防挂机请求不使用此值。',
    },
    'zh_tw': {
        '': '選擇舊版持續前進與轉頭、隨機互動或自訂防掛機行為。',
        'randomInteraction.interaction': '允許隨機選擇的互動：跳躍、揮手、切換欄位、水平轉向、垂直轉向或隨機移動。',
        'randomInteraction.delay': '每次隨機互動後等待的遊戲刻數範圍；隨機移動也持續此範圍內的時間。',
        'custom.rotate.enabled': '按設定間隔發起防掛機轉向。',
        'custom.rotate.ignoreOpenInventory': '開啟物品欄時仍允許提交防掛機轉向。',
        'custom.rotate.delay': '每次發起自訂轉向前等待的遊戲刻數。',
        'custom.rotate.angle': '在目前伺服器水平朝向上增加此角度；垂直朝向另外加入 -5 至 5 度隨機偏移。',
        'custom.swing.enabled': '按設定間隔揮動主手。',
        'custom.swing.delay': '未在揮手時，等待這些遊戲刻數再開始下一次揮手。',
        'custom.jump': '站在地面時自動跳躍。',
        'custom.move': '持續按下前進鍵。',
        'custom.rotate.rotations': '設定自訂防掛機轉向的平滑過程、移動修正及返回可見視角的規則。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.enabled': '存在目標實體時使用準星加速度；防掛機的轉向請求沒有目標實體。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.coefDistance': '距離對加速度的影響係數；防掛機轉向採用零目標距離。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.yawCrosshairAccel': '準星指向目標實體時使用的水平加速度；無目標實體的防掛機請求不使用此值。',
        'custom.rotate.rotations.angleSmooth.acceleration.dynamicAccel.pitchCrosshairAccel': '準星指向目標實體時使用的垂直加速度；無目標實體的防掛機請求不使用此值。',
    },
}


def main():
    for locale in ['zh_cn', 'zh_tw']:
        path = FOLDER / f'display_{locale}.json'
        display = json.loads(path.read_text(encoding='utf-8'))
        fields = [('descriptions', locale)] + ([('englishDescriptions', 'en')] if locale == 'zh_cn' else [])
        for field, language in fields:
            descriptions = display[field]
            descriptions.update({key.replace(OLD, ROTATIONS): text
                                 for key, text in list(descriptions.items()) if key.startswith(OLD)})
            descriptions.update({PREFIX.rstrip('.') + ('.' + suffix if suffix else '') + '.description': text
                                 for suffix, text in HELP[language].items()})
        path.write_text(json.dumps(display, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        print(f'{locale}: {len(display["descriptions"])} owned descriptions')


if __name__ == '__main__':
    main()
