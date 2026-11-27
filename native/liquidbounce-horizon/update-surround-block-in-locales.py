#!/usr/bin/env python3
from pathlib import Path
import argparse, json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'surround':('自动环绕','自動環繞','Builds a protective surround around the player.'),
 'surround.features':('功能','功能','Surround planning features.'), 'surround.disableOn':('禁用条件','停用條件','Conditions that disable Surround.'),
 'surround.instant':('即时补放','即時補放','Replace broken surround blocks immediately.'), 'surround.filter':('筛选方式','篩選方式','Choose the block filter mode.'), 'surround.blocks':('方块','方塊','Blocks used by Surround.'),
 'blockIn':('自封','自封','Build blocks around and above the player.'), 'blockIn.disableOn':('禁用条件','停用條件','Conditions that disable BlockIn.'),
 'blockIn.avoidBrokenBlocks':('避开破坏方块','避開破壞方塊','Avoid positions recently broken by the player.'), 'blockIn.brokenBlockWindow':('破坏记录时长','破壞記錄時長','How long recently broken positions remain blocked.'),
 'blockIn.placeOrder':('放置顺序','放置順序','Order used for BlockIn placements.'), 'blockIn.filter':('筛选方式','篩選方式','Choose the block filter mode.'), 'blockIn.blocks':('方块','方塊','Blocks used by BlockIn.'),
}
def update(folder):
 for lang,index in [('zh_cn',0),('zh_tw',1)]:
  path=folder/f'display_{lang}.json'; data=json.loads(path.read_text(encoding='utf8'))
  for module,label in [('Surround','surround'),('BlockIn','blockIn')]:
   values={k.split('.',1)[1] if k.startswith(label+'.') else '':v[index] for k,v in TEXTS.items() if k==label or k.startswith(label+'.')}
   descriptions={('description' if k==label else k[len(label)+1:]+'.description'):v[2] for k,v in TEXTS.items() if k==label or k.startswith(label+'.')}
   put_module(data,'liquidbounce.module.'+('surround' if module=='Surround' else 'blockIn'),module,{**values,**descriptions},descriptions)
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(ap.parse_args().folder)
