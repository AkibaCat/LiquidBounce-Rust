#!/usr/bin/env python3
"""HoleFiller Chinese captions and full inherited placement settings."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.holeFiller'
LF=runpy.run_path(str(HERE/'update-liquid-filler-locales.py'))
LABELS={'HoleFiller':('自动填洞','自動填洞'),'Features':('功能','功能'),'Area':('目标脚下检查范围','目標腳下檢查範圍'),'Filter':('方块筛选方式','方塊篩選方式'),'Blocks':('填洞方块','填洞方塊'),'Placing':('方块放置器','方塊放置器'),'Smart':('智能填洞','智慧填洞'),'PreventSelfFill':('防止填到自身','防止填到自身'),'OnlyWhenSelfInHole':('仅在自身位于洞中时','僅在自身位於洞中時'),'CheckMovement':('检查目标移动角度','檢查目標移動角度'),'Only1x1':('仅填充单格洞','僅填充單格洞')}
def entries():
 texts,scoped=LF['entries']();texts={k.replace('placer.','placing.',1):v for k,v in texts.items()if k.startswith('placer.')};scoped={k.replace('placer','placing',1):v for k,v in scoped.items()if k=='placer'or k.startswith('placer.')}
 texts.update({
 'description':('Automatically fills tracked holes with selected blocks.','使用选定的方块自动填充已跟踪的洞位。','使用選定的方塊自動填充已追蹤的洞位。'),
 'features.description':('Enable smart collection, self-fill protection, movement checks and hole-size limits.','选择智能目标收集、自身保护、移动角度检查和洞位大小限制。','選擇智慧目標收集、自身保護、移動角度檢查和洞位大小限制。'),
 'features.smart.description':('Collect holes around valid attack targets and limit them to available block items.','收集可攻击目标附近的洞位，并按可用方块物品数量限制填充。','收集可攻擊目標附近的洞位，並依可用方塊物品數量限制填充。'),
 'features.preventSelfFill.description':('Allow nearby holes only when higher than you or when you are already in a tracked hole.','附近洞位仅在高于自身或自身已处于受跟踪洞位时才允许填充。','附近洞位僅在高於自身或自身已處於受追蹤洞位時才允許填充。'),
 'features.onlyWhenSelfInHole.description':('Clear the placement queue when you are not in an eligible tracked hole.','自身未处于符合筛选条件的洞位时，清空放置队列。','自身未處於符合篩選條件的洞位時，清空放置佇列。'),
 'features.checkMovement.description':('Apply the original movement-angle check in smart mode.','在智能模式中应用原版的移动角度判定。','在智慧模式中套用原版的移動角度判定。'),
 'features.only1x1.description':('Ignore two-cell and four-cell holes.','忽略占两格和四格的洞位。','忽略佔兩格和四格的洞位。'),
 'area.description':('Expand the area around each entity foot position by this many blocks in all axes.','以实体脚下位置为中心，沿各坐标轴扩展此方块数来检查洞位。','以實體腳下位置為中心，沿各座標軸擴展此方塊數來檢查洞位。'),
 'filter.description':('Choose whether the block list permits or excludes blocks.','选择方块列表作为白名单还是黑名单。','選擇方塊清單作為白名單或黑名單。'),
 'blocks.description':('Blocks considered by the whitelist or blacklist; obsidian is selected by default.','白名单或黑名单使用的方块列表，默认选中黑曜石。','白名單或黑名單使用的方塊清單，預設選取黑曜石。')})
 for name in['Features','Area','Filter','Blocks','Placing']:scoped[name[:1].lower()+name[1:]]=LABELS[name]
 for name in['Smart','PreventSelfFill','OnlyWhenSelfInHole','CheckMovement','Only1x1']:scoped['features.'+name]=LABELS[name]
 return texts,scoped
def update(folder):
 texts,scoped=entries()
 for locale,index in[('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));put_module(data,PREFIX,'HoleFiller',{'':LABELS['HoleFiller'][index-1],**{k:v[index]for k,v in texts.items()}},{k:v[0]for k,v in texts.items()});labels=data.setdefault('labels',{})
  for k,v in LABELS.items():labels.setdefault(k,v[index-1])
  for k,v in scoped.items():labels[PREFIX+'.'+k]=v[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
