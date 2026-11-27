#!/usr/bin/env python3
"""ProjectilePuncher and complete combat Rotations Chinese display resources."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
LABELS = {
    'Projectiles': ('弹射物种类', '彈射物種類'),
    'Fireball': ('大型火球', '大型火球'), 'ShulkerBullet': ('潜影弹', '界伏彈'),
    'WindCharge': ('风弹', '風彈'), 'OnlyInAir': ('仅在空中', '僅在空中'),
    'Interpolation': ('插值', '插值'), 'AI': ('AI', 'AI'),
    'HorizontalSpeed': ('水平速度', '水平速度'), 'VerticalSpeed': ('垂直速度', '垂直速度'),
    'DirectionChangeFactor': ('方向变化系数', '方向變化係數'),
    'ShortStop': ('短暂停顿', '短暫停頓'), 'Duration': ('持续时间', '持續時間'),
    'Fail': ('瞄准失误', '瞄準失誤'), 'Rate': ('概率', '機率'), 'Factor': ('偏移系数', '偏移係數'),
    'StrengthHorizontal': ('水平偏移强度', '水平偏移強度'),
    'StrengthVertical': ('垂直偏移强度', '垂直偏移強度'),
    'TransitionInDuration': ('失误持续时间', '失誤持續時間'),
    'Model': ('模型', '模型'), 'Correction': ('输出校正', '輸出校正'),
    'OutputMultiplier': ('输出倍率', '輸出倍率'), 'Yaw': ('水平角度', '水平角度'),
    'Pitch': ('垂直角度', '垂直角度'),
}

# Original source names are converted with lower-first, including AI -> aI.
TEXTS = {
    '': ('Punch incoming large fireballs, shulker bullets and selected wind charges.', '反击接近的大型火球、潜影弹及选中的风弹。', '反擊接近的大型火球、界伏彈及選取的風彈。'),
    'Clicker': ('Independent attack click plan shared with other attack clickers and physical presses.', '独立的攻击点击计划，与其他攻击连点器和实际按键协调。', '獨立的攻擊點擊計畫，與其他攻擊連點器和實際按鍵協調。'),
    'Clicker.Technique': ('Human varies the click rhythm; Constant uses the upper CPS endpoint. Default Human.', '拟人模式改变点击节奏；恒定模式使用 CPS 上限。默认拟人。', '擬人模式改變點擊節奏；固定模式使用 CPS 上限。預設擬人。'),
    'Clicker.CPS': ('Clicks per second, sampled by the selected technique. Default 11–14.', '所选方式使用的每秒点击次数，默认 11–14。', '所選方式使用的每秒點擊次數，預設 11–14。'),
    'Clicker.MaxPerTick': ('Limit scheduled clicks in one game tick. Default 2.', '限制一个游戏刻内计划的点击次数，默认 2。', '限制一個遊戲刻內計畫的點擊次數，預設 2。'),
    'Clicker.MissCooldown': ('Wait for the real miss timer before attacking. Default true; this clicker has no item-cooldown setting.', '攻击前等待实际打空冷却结束。默认开启；此连点器没有物品冷却设置。', '攻擊前等待實際揮空冷卻結束。預設開啟；此連點器沒有物品冷卻設定。'),
    'Range': ('Maximum attack and one-tick prediction range. Default 3 blocks.', '攻击和下一刻位置预测的最大范围，默认 3 格。', '攻擊與下一刻位置預測的最大範圍，預設 3 格。'),
    'SwingMode': ('Control local and server swings. Default DoNotHide.', '控制本地和服务端挥手，默认不隐藏。', '控制本機和伺服器揮手，預設不隱藏。'),
    'IgnoreOpenInventory': ('Permit requested rotations while the inventory is open. Default true.', '库存打开时仍允许所请求的旋转，默认开启。', '物品欄開啟時仍允許所請求的旋轉，預設開啟。'),
    'Projectiles': ('Select Fireball, ShulkerBullet and WindCharge. Default Fireball and ShulkerBullet.', '选择大型火球、潜影弹和风弹。默认大型火球及潜影弹。', '選擇大型火球、界伏彈和風彈。預設大型火球及界伏彈。'),
    'OnlyInAir': ('Acquire targets only while the player is not on the ground. Default false.', '仅玩家离地时获取目标，默认关闭。', '僅玩家離地時取得目標，預設關閉。'),
    'FOV': ('Maximum angle between the local view and the projectile physical box. Default 180 degrees.', '本地视线与弹射物实际包围盒之间的最大角度，默认 180 度。', '本機視線與彈射物實際邊界框之間的最大角度，預設 180 度。'),
    'Rotations': ('Aim using the full combat rotation processor chain.', '使用完整战斗旋转处理链瞄准。', '使用完整戰鬥旋轉處理鏈瞄準。'),
    'Rotations.AngleSmooth': ('Choose Linear, Sigmoid, Interpolation, Acceleration or AI. Default Linear.', '选择线性、S 型、插值、加速度或 AI。默认线性。', '選擇線性、S 型、插值、加速度或 AI。預設線性。'),
    'Rotations.ShortStop': ('Temporarily limit rotation to a tiny movement.', '短暂将旋转限制为微小移动。', '短暫將旋轉限制為微小移動。'),
    'Rotations.ShortStop.Enabled': ('Enable short stops. Default false.', '启用短暂停顿，默认关闭。', '啟用短暫停頓，預設關閉。'),
    'Rotations.ShortStop.Rate': ('Chance of starting a stop during each processed rotation. Default 3 percent.', '每次处理旋转时开始停顿的概率，默认 3%。', '每次處理旋轉時開始停頓的機率，預設 3%。'),
    'Rotations.ShortStop.Duration': ('Number of processed rotations to stop. Default 1–2 ticks.', '停顿持续的旋转处理次数，默认 1–2 刻。', '停頓持續的旋轉處理次數，預設 1–2 刻。'),
    'Rotations.Fail': ('Occasionally offset aim using previous and server rotations.', '偶尔根据上一旋转和服务端旋转偏移瞄准。', '偶爾依上一旋轉和伺服器旋轉偏移瞄準。'),
    'Rotations.Fail.Enabled': ('Enable deliberate aim failures. Default false.', '启用刻意瞄准失误，默认关闭。', '啟用刻意瞄準失誤，預設關閉。'),
    'Rotations.Fail.Rate': ('Chance per game tick of starting a failure. Default 3 percent.', '每个游戏刻开始失误的概率，默认 3%。', '每個遊戲刻開始失誤的機率，預設 3%。'),
    'Rotations.Fail.Factor': ('Scale the previous-to-server rotation difference. Default 0.04.', '上一旋转与服务端旋转之差的倍率，默认 0.04。', '上一旋轉與伺服器旋轉之差的倍率，預設 0.04。'),
    'Rotations.Fail.StrengthHorizontal': ('Random horizontal aim shift. Default 5–10 degrees.', '随机水平瞄准偏移，默认 5–10 度。', '隨機水平瞄準偏移，預設 5–10 度。'),
    'Rotations.Fail.StrengthVertical': ('Random vertical aim shift. Default 0–2 degrees.', '随机垂直瞄准偏移，默认 0–2 度。', '隨機垂直瞄準偏移，預設 0–2 度。'),
    'Rotations.Fail.TransitionInDuration': ('Keep the sampled failure for this many ticks. Default 1–4.', '维持此次随机失误的刻数，默认 1–4。', '維持此次隨機失誤的刻數，預設 1–4。'),
    'Rotations.AngleSmooth.AI': ('Use the selected model when the learning engine is available; this native runtime uses the original Interpolation fallback.', '学习引擎可用时使用选中的模型；当前原生运行时使用源码规定的插值回退。', '學習引擎可用時使用選取的模型；目前原生執行環境使用原始碼規定的插值回退。'),
    'Rotations.AngleSmooth.AI.Model': ('Select the original built-in regression model. Default 21KC11KP.', '选择原有内置回归模型，默认 21KC11KP。', '選擇原有內建迴歸模型，預設 21KC11KP。'),
    'Rotations.AngleSmooth.AI.Correction': ('Correction after model output: Interpolation, Linear or None. Retained while AI falls back.', '模型输出后的校正：插值、线性或无。AI 回退时保留此设置。', '模型輸出後的校正：插值、線性或無。AI 回退時保留此設定。'),
    'Rotations.AngleSmooth.AI.Correction.Linear.HorizontalTurnSpeed': ('Horizontal model-correction speed. Default 5 degrees.', '模型校正的水平速度，默认 5 度。', '模型校正的水平速度，預設 5 度。'),
    'Rotations.AngleSmooth.AI.Correction.Linear.VerticalTurnSpeed': ('Vertical model-correction speed. Default 5 degrees.', '模型校正的垂直速度，默认 5 度。', '模型校正的垂直速度，預設 5 度。'),
    'Rotations.AngleSmooth.AI.OutputMultiplier': ('Scale the two model outputs before correction.', '在校正之前缩放模型的两项输出。', '在校正之前縮放模型的兩項輸出。'),
    'Rotations.AngleSmooth.AI.OutputMultiplier.Yaw': ('Model horizontal-output multiplier. Default 1.5.', '模型水平输出倍率，默认 1.5。', '模型水平輸出倍率，預設 1.5。'),
    'Rotations.AngleSmooth.AI.OutputMultiplier.Pitch': ('Model vertical-output multiplier. Default 1.', '模型垂直输出倍率，默认 1。', '模型垂直輸出倍率，預設 1。'),
}
for prefix, correction in [('Rotations.AngleSmooth.Interpolation', False), ('Rotations.AngleSmooth.AI.Correction.Interpolation', True)]:
    TEXTS[prefix] = ('Interpolate toward the target using angle and direction change.', '按角度和方向变化向目标插值。', '依角度和方向變化向目標插值。')
    for name, en, cn, tw in [
        ('HorizontalSpeed', f'Horizontal interpolation speed. Default {"2–5" if correction else "80–85"} percent.', f'水平插值速度，默认 {"2–5" if correction else "80–85"}%。', f'水平插值速度，預設 {"2–5" if correction else "80–85"}%。'),
        ('VerticalSpeed', f'Vertical interpolation speed. Default {"2–5" if correction else "20–25"} percent.', f'垂直插值速度，默认 {"2–5" if correction else "20–25"}%。', f'垂直插值速度，預設 {"2–5" if correction else "20–25"}%。'),
        ('DirectionChangeFactor', 'Additional factor for changing target direction. Default 95–100 percent.', '目标方向变化时的附加系数，默认 95–100%。', '目標方向變化時的附加係數，預設 95–100%。'),
        ('Midpoint', 'Threshold between sigmoid and Bezier interpolation. Default 0.35.', 'S 型与贝塞尔插值的切换阈值，默认 0.35。', 'S 型與貝茲插值的切換門檻，預設 0.35。'),
    ]:
        TEXTS[f'{prefix}.{name}'] = (en, cn, tw)


def description_key(path):
    suffix = '.'.join(part[:1].lower() + part[1:] for part in path.split('.'))
    return 'liquidbounce.module.projectilePuncher' + ('.' + suffix if suffix else '') + '.description'


def update(data, index):
    data['moduleNames']['ProjectilePuncher'] = ('弹射物反击', '彈射物反擊')[index - 1]
    data['labels'].update({k: v[index - 1] for k, v in LABELS.items()})
    for field in ['descriptions', 'englishDescriptions']:
        if field not in data:
            continue
        for key, text in list(data[field].items()):
            old = 'liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
            if key.startswith(old):
                data[field][key.replace(old, 'liquidbounce.module.projectilePuncher.rotations')] = text
    for setting, text in TEXTS.items():
        key = description_key(setting)
        data['descriptions'][key] = text[index]
        if index == 1:
            data['englishDescriptions'][key] = text[0]
    return data


def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        data = update(json.loads(path.read_text(encoding='utf8')), index)
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
        print(locale, len(TEXTS), 'ProjectilePuncher descriptions plus complete shared rotation descriptions')


if __name__ == '__main__':
    main()
