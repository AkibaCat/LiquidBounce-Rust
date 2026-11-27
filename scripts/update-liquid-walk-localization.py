#!/usr/bin/env python3
"""Simplified/traditional LiquidWalk display translations."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
TEXTS={
 'liquidWalk':('Allows you to walk on water and lava.','在水面和熔岩表面行走。','在水面和熔岩表面行走。'),
 'liquidWalk.mode':('Vanilla adds surface collision and swim input. NCP and Verus adjust movement packets; Vulcan boosts upward in water or bubble columns.','原版模式添加液面碰撞并自动游起；NCP 与 Verus 调整移动数据包；Vulcan 在水或气泡柱中向上加速。','原版模式加入液面碰撞並自動游起；NCP 與 Verus 調整移動封包；Vulcan 在水或氣泡柱中向上加速。'),
 'liquidWalk.mode.vulcan-2.9.1.motion':('Upward velocity in water or bubble columns. Default 0.8; this mode does not boost in lava.','水或气泡柱中的向上速度，默认 0.8；此模式不会在熔岩中上浮。','水或氣泡柱中的向上速度，預設 0.8；此模式不會在熔岩中上浮。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['LiquidWalk']=('水上行走','水上行走')[index-1]
  for name,texts in TEXTS.items():
   key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=texts[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=texts[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
