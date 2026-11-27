#!/usr/bin/env python3
"""BedDefender full Chinese settings, inactive modes and manual tracking notices."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.bedDefender'
LF=runpy.run_path(str(HERE/'update-liquid-filler-locales.py'))
BP=runpy.run_path(str(HERE/'update-bed-plates-locales.py'))
LABELS={'BedDefender':('床防护','床防護'),'SelfBed':('自身床识别','自身床識別'),'Place':('方块放置器','方塊放置器'),'MaxLayers':('最大防护层数','最大防護層數'),'AllowChests':('允许放置箱子','允許放置箱子'),'RequiresSneak':('需要潜行','需要潛行')}
def entries():
 texts,scoped=LF['entries']();texts={k.replace('placer.','place.',1):v for k,v in texts.items()if k.startswith('placer.')};scoped={k.replace('placer','place',1):v for k,v in scoped.items()if k=='placer'or k.startswith('placer.')}
 for key,value in BP['TEXTS'].items():
  if key.startswith('ignoreSelfBed.'):texts[key.replace('ignoreSelfBed','selfBed',1)]=value
 texts.update({
  'description':('Defend the closest matching bed with ordered layers of sturdy blocks.','使用按层规划的高耐挖方块，防护最近且符合自身床识别条件的床。','使用依層規劃的高耐挖方塊，防護最近且符合自身床識別條件的床。'),
  'maxLayers.description':('Number of outward and upward defense layers around both bed parts.','围绕床的两部分向外、向上构建的防护层数。','圍繞床的兩部分向外、向上建造的防護層數。'),
  'allowChests.description':('Allow normal, trapped, ender and copper chest items in addition to full collision blocks.','除完整碰撞方块外，还允许使用普通箱子、陷阱箱、末影箱和铜箱。','除完整碰撞方塊外，亦允許使用一般箱子、陷阱箱、終界箱和銅箱。'),
  'requiresSneak.description':('Clear placement targets while the player is not sneaking.','玩家未潜行时清空待放置目标。','玩家未潛行時清空待放置目標。'),
  'selfBed.description':('Choose which beds should be defended.','选择应当防护哪些床。','選擇應當防護哪些床。'),
  'selfBed.none.description':('Defend the closest bed without identifying a self bed.','无需识别自身床，直接防护最近的床。','無須識別自身床，直接防護最近的床。')})
 for name,value in LABELS.items():scoped[name[:1].lower()+name[1:]]=value
 for name in ['Color','Slots','Feet','Legs','Chest','Head','Loose','SpawnLocation','BedDistance','Manual','Track','Untrack']:
  if name in BP['LABELS']:scoped['selfBed.'+name]=BP['LABELS'][name]
 for path,name in [('selfBed.color.slots.Chest','Chest'),('selfBed.color.slots.Head','Head'),('selfBed.color.slots.Legs','Legs'),('selfBed.color.slots.Feet','Feet')]:scoped[path]=BP['LABELS'][name]
 return texts,scoped

def update(folder):
 texts,scoped=entries()
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  put_module(data,PREFIX,'BedDefender',{'':LABELS['BedDefender'][index-1],**{k:v[index]for k,v in texts.items()}},{k:v[0]for k,v in texts.items()})
  for k,v in BP['TEXTS'].items():
   if k.startswith('notifications.'):data['descriptions'][PREFIX+'.'+k]=v[index]
  labels=data.setdefault('labels',{})
  for k,v in LABELS.items():labels.setdefault(k,v[index-1])
  for k,v in scoped.items():labels[PREFIX+'.'+k]=v[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
