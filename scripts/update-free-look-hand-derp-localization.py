#!/usr/bin/env python3
"""Owned EN/CN/TW help for all pinned FreeLook and HandDerp leaves."""
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
HELP={
    'freeLook.perspective': (
        'View the player from the front or back while moving the camera independently of the player\'s facing direction.',
        '从玩家前方或后方观察，移动镜头时保持玩家实际朝向。',
        '從玩家前方或後方觀察，移動鏡頭時保持玩家實際朝向。'),
    'freeLook.senseBoost': (
        'Multiply FreeLook mouse rotation by this factor after the game\'s mouse sensitivity, inversion and zoom scaling.',
        '在游戏鼠标灵敏度、反转和缩放处理后，将自由视角的鼠标转动乘以此倍率。',
        '在遊戲滑鼠靈敏度、反轉和縮放處理後，將自由視角的滑鼠轉動乘以此倍率。'),
    'freeLook.noPitchLimit': (
        'Allow the camera to rotate vertically beyond -90 through 90 degrees; disabling this limits its pitch on the next mouse input.',
        '允许镜头的垂直转动超出 -90 至 90 度；关闭后会在下一次鼠标输入时限制俯仰角。',
        '允許鏡頭的垂直轉動超出 -90 至 90 度；關閉後會在下一次滑鼠輸入時限制俯仰角。'),
    'handDerp.silent': (
        'Cancel an entire received entity-data packet when it contains an index equal to the actual main-hand option ID (0 or 1), matching the original module.',
        '收到实体数据包时，若含有与实际主手选项编号（0 或 1）相同的索引，则取消整个数据包，沿用原模块的匹配规则。',
        '收到實體資料封包時，若含有與實際主手選項編號（0 或 1）相同的索引，則取消整個封包，沿用原模組的比對規則。'),
    'handDerp.mode.delay.delay': (
        'Wait this many game ticks between sending alternating main-hand preferences to the server; the local main-hand option stays unchanged.',
        '向服务器交替发送主手偏好之间等待的游戏刻数；本地主手设置保持不变。',
        '向伺服器交替傳送主手偏好之間等待的遊戲刻數；本地主手設定保持不變。'),
}


def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json'
        display=json.loads(path.read_text(encoding='utf-8'))
        for key,texts in HELP.items():
            key='liquidbounce.module.'+key+'.description'
            display['descriptions'][key]=texts[index]
            if locale=='zh_cn':display['englishDescriptions'][key]=texts[0]
        path.write_text(json.dumps(display,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        print(f'{locale}: {len(display["descriptions"])} owned descriptions')


if __name__=='__main__':main()
