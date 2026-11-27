#!/usr/bin/env python3
"""Merge SpearKill setting help and English fallback; --check makes no writes."""
from pathlib import Path
import argparse,json
ROOT=Path(__file__).resolve().parents[2]
HELP={
    'en_us':{
        'description':'Automatically attacks enemies using a charged spear.',
        'maxTargetDistance.description':'Maximum distance to find a living target along the view direction.',
        'maxSpeed.description':'Maximum movement speed per tick during the charge attack and return.',
        'preview.description':'Show the current target bounding box.',
        'preview.enabled.description':'Draw the current target preview.',
        'preview.fillColor.description':'Fill color and transparency of the target preview.',
        'preview.outlineColor.description':'Outline color and transparency of the target preview.',
    },
    'zh_cn':{
        'description':'使用蓄力长矛自动攻击敌人。',
        'maxTargetDistance.description':'沿视线寻找存活目标的最大距离。',
        'maxSpeed.description':'蓄力攻击及返回时每刻移动速度的上限。',
        'preview.description':'显示当前目标的碰撞箱。',
        'preview.enabled.description':'绘制当前目标预览。',
        'preview.fillColor.description':'目标预览的填充颜色与透明度。',
        'preview.outlineColor.description':'目标预览的轮廓颜色与透明度。',
    },
    'zh_tw':{
        'description':'使用蓄力長矛自動攻擊敵人。',
        'maxTargetDistance.description':'沿視線尋找存活目標的最大距離。',
        'maxSpeed.description':'蓄力攻擊及返回時每刻移動速度的上限。',
        'preview.description':'顯示目前目標的碰撞箱。',
        'preview.enabled.description':'繪製目前目標預覽。',
        'preview.fillColor.description':'目標預覽的填色與透明度。',
        'preview.outlineColor.description':'目標預覽的輪廓顏色與透明度。',
    },
}
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();pending={}
    for lang in ('zh_cn','zh_tw'):
        path=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';data=json.loads(path.read_text(encoding='utf8'))
        data['descriptions'].update({f'liquidbounce.module.spearKill.{key}':value for key,value in HELP[lang].items()})
        if lang=='zh_cn':data['englishDescriptions'].update({f'liquidbounce.module.spearKill.{key}':value for key,value in HELP['en_us'].items()})
        pending[path]=json.dumps(data,ensure_ascii=False,indent=2)+'\n'
    if args.check:print('Validated SpearKill help in both Chinese variants and English fallback; no writes')
    else:
        for path,content in pending.items():path.write_text(content,encoding='utf8')
        print('Merged SpearKill localization')
if __name__=='__main__':main()
