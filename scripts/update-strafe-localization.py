#!/usr/bin/env python3
"""Owned complete Strafe setting help, preserving stable configuration keys."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
HELP={
    'strengthInAir':(
        'Blend horizontal self-movement toward the untransformed input direction while airborne. One fully redirects it at the same horizontal speed; zero leaves movement unchanged.',
        '空中自主移动时，将水平运动按此强度转向修正前的输入方向。设为 1 时，以相同水平速度完全转向；设为 0 时保持原运动。',
        '空中自主移動時，將水平運動依此強度轉向修正前的輸入方向。設為 1 時，以相同水平速度完全轉向；設為 0 時維持原運動。'),
    'strengthOnGround':(
        'Blend horizontal self-movement toward the untransformed input direction while on the ground. One fully redirects it at the same horizontal speed; zero leaves movement unchanged.',
        '地面自主移动时，将水平运动按此强度转向修正前的输入方向。设为 1 时，以相同水平速度完全转向；设为 0 时保持原运动。',
        '地面自主移動時，將水平運動依此強度轉向修正前的輸入方向。設為 1 時，以相同水平速度完全轉向；設為 0 時維持原運動。'),
    'strictMovement':(
        'Set horizontal self-movement to zero when the player move vector is zero. The selected ground or air strength must be nonzero; vertical movement is preserved.',
        '玩家移动输入向量为零时，将自主移动的水平分量归零。当前地面或空中强度须不为 0；垂直运动保留。',
        '玩家移動輸入向量為零時，將自主移動的水平分量歸零。目前地面或空中強度須不為 0；垂直運動保留。'),
}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
    path=FOLDER/f'display_{locale}.json'
    data=json.loads(path.read_text(encoding='utf-8'))
    for key,texts in HELP.items():
        key=f'liquidbounce.module.strafe.{key}.description'
        data['descriptions'][key]=texts[index]
        if locale=='zh_cn': data['englishDescriptions'][key]=texts[0]
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'{locale}: {len(data["descriptions"])} owned descriptions')
