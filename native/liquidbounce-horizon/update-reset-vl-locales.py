#!/usr/bin/env python3
"""Merge ResetVL descriptions and complete setting help; run at integration."""
from pathlib import Path
import argparse, json
ROOT=Path(__file__).resolve().parents[2]
BASE='liquidbounce.module.resetVL.'
HELP={
    'en_us':{
        'times.description':'Number of grounded upward/downward sequences before the module disables itself and reports success. Each sequence sets all velocity axes to (0, 0.1, 0), waits one game tick, then sets (0, -0.1, 0). Partial progress survives disable and re-enable.',
        'timer.description':'Client timer speed requested at the user safety priority during each started tick handler, including while airborne. Higher-priority requests take precedence. The module applies no server packet or movement-mode replacement.',
    },
    'zh_cn':{
        'times.description':'完成指定次数的落地上升／下降序列后自动关闭并提示成功。每次先把完整速度设为 (0, 0.1, 0)，等待一个游戏 tick，再设为 (0, -0.1, 0)。关闭后重新开启会保留尚未完成的计数。',
        'timer.description':'每次开始执行 tick 处理器时，以用户安全优先级请求此客户端运行速度；在空中也会请求。更高优先级的请求优先生效。默认 2 倍，界面范围 0.1～10 倍。',
    },
    'zh_tw':{
        'times.description':'完成指定次數的落地上升／下降序列後自動關閉並提示成功。每次先將完整速度設為 (0, 0.1, 0)，等待一個遊戲 tick，再設為 (0, -0.1, 0)。關閉後重新開啟會保留尚未完成的計數。',
        'timer.description':'每次開始執行 tick 處理器時，以使用者安全優先級請求此客戶端運行速度；在空中也會請求。較高優先級的請求優先生效。預設 2 倍，介面範圍 0.1～10 倍。',
    },
}

def update(language, target):
    data=json.loads(target.read_text(encoding='utf8'))
    source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
    data['descriptions'][BASE+'description']=source[BASE+'description']
    data['descriptions'].update({BASE+k:v for k,v in HELP[language].items()})
    data['labels'].update({'Times':'次数' if language=='zh_cn' else '次數'})
    if language=='zh_cn':
        source=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
        data['englishDescriptions'][BASE+'description']=source[BASE+'description']
        data['englishDescriptions'].update({BASE+k:v for k,v in HELP['en_us'].items()})
    target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--directory',default='native/liquidbounce-interop/resources/localization');a=parser.parse_args()
    for language in ['zh_cn','zh_tw']:update(language,ROOT/a.directory/f'display_{language}.json')

if __name__=='__main__':main()
