#!/usr/bin/env python3
"""Describe the three formula coefficients, rather than labelling them axes."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
KEY='liquidbounce.module.extendedFirework.mode.custom.velocityMultiplier.description'
TEXTS=(
    'Three boost-formula coefficients, not separate world axes: x is added look-direction acceleration, y is target look-direction speed, and z is the correction fraction toward that target. Defaults are 0.1, 1.65 and 0.5; the formula applies to every velocity axis.',
    '烟花助推公式的三个系数：x 为沿视线方向额外增加的加速度，y 为沿视线方向的目标速度，z 为向目标速度修正的比例。默认值为 0.1、1.65、0.5；这套公式作用于每个速度轴，并非分别设置世界坐标轴。',
    '煙火助推公式的三個係數：x 為沿視線方向額外增加的加速度，y 為沿視線方向的目標速度，z 為向目標速度修正的比例。預設值為 0.1、1.65、0.5；這套公式套用至每個速度軸，並非分別設定世界座標軸。')
for locale,index in [('zh_cn',1),('zh_tw',2)]:
    path=FOLDER/f'display_{locale}.json'
    data=json.loads(path.read_text(encoding='utf-8'))
    data['descriptions'][KEY]=TEXTS[index]
    if locale=='zh_cn': data['englishDescriptions'][KEY]=TEXTS[0]
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'{locale}: {len(data["descriptions"])} owned descriptions')
