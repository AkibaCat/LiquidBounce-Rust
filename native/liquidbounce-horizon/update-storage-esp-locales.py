"""Complete nested StorageESP help; writes only the explicitly supplied dictionary copies."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
BASE=runpy.run_path(str(HERE/'update-breadcrumbs-locales.py'))
TYPES=['Chest','TrappedChest','Barrel','EnderChest','Furnace','BrewingStand','Dispenser','Dropper','Hopper','ShulkerBox','Pot','Bookshelf','Shelf']
LABELS=dict(zip(TYPES,['箱子','陷阱箱','木桶','末影箱','熔炉','酿造台','发射器','投掷器','漏斗','潜影盒','饰纹陶罐','雕纹书架','置物架']))
LABELS_TW=dict(zip(TYPES,['儲物箱','陷阱儲物箱','木桶','終界箱','熔爐','釀造台','發射器','投擲器','漏斗','界伏盒','裝飾陶罐','浮雕書櫃','置物架']))
LABELS.update({'StorageESP':'存储透视','ChestESP':'箱子透视','Box':'方框','Glow':'辉光','Vanilla':'原版','RequiresChestStealer':'需要容器窃取','MergeAdjacent':'合并相邻形状','DistanceFade':'距离渐隐','NearStart':'近端起点','NearEnd':'近端终点','FarStart':'远端起点','FarEnd':'远端终点','Tracers':'示踪线'})
LABELS_TW.update({'StorageESP':'儲存透視','ChestESP':'儲物箱透視','Box':'方框','Glow':'輝光','Vanilla':'原版','RequiresChestStealer':'需要容器竊取','MergeAdjacent':'合併相鄰形狀','DistanceFade':'距離漸隱','NearStart':'近端起點','NearEnd':'近端終點','FarStart':'遠端起點','FarEnd':'遠端終點','Tracers':'追蹤線'})
HELP={'Mode':'选择实际形状方框或真实模型辉光。','Mode.Box.Outline':'绘制形状边框，透明度固定 100/255；填充固定 50/255。','Mode.Glow.Style':'原版边缘检测或九种完整后处理样式，默认柔光。','RequiresChestStealer':'仅在容器窃取实际运行时显示。','MergeAdjacent':'按同类存储的六方向相邻关系合并体素形状，去除内部面和边。','DistanceFade.NearStart':'近端渐入起点；不得大于近端终点，范围 0～512 方块。','DistanceFade.NearEnd':'近端渐入终点；限制在近端起点与远端起点之间。','DistanceFade.FarStart':'远端渐出起点；限制在近端终点与远端终点之间。','DistanceFade.FarEnd':'远端渐出终点；不得小于远端起点。模型与示踪线也以此距离严格筛选。'}
HELP_TW={'Mode':'選擇實際形狀方框或真實模型輝光。','Mode.Box.Outline':'繪製形狀邊框，透明度固定 100/255；填滿固定 50/255。','Mode.Glow.Style':'原版邊緣偵測或九種完整後製樣式，預設柔光。','RequiresChestStealer':'僅在容器竊取實際執行時顯示。','MergeAdjacent':'依同類儲存的六方向相鄰關係合併體素形狀，移除內部面與邊。','DistanceFade.NearStart':'近端漸入起點；不得大於近端終點，範圍 0～512 方塊。','DistanceFade.NearEnd':'近端漸入終點；限制在近端起點與遠端起點之間。','DistanceFade.FarStart':'遠端漸出起點；限制在近端終點與遠端終點之間。','DistanceFade.FarEnd':'遠端漸出終點；不得小於遠端起點。模型與追蹤線也以此距離嚴格篩選。'}
def key(p):return 'liquidbounce.module.storageESP.'+'.'.join(n[0].lower()+n[1:]for n in p.split('.'))
def descriptions(language):
 source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'));result={'liquidbounce.module.storageESP.description':source['liquidbounce.module.storageESP.description']}
 result.update({key(p)+'.description':v for p,v in(HELP_TW if language=='zh_tw'else HELP).items()})
 for name in TYPES:
  for field,help in [('Enabled','显示此类存储。'),('Color','此类存储的 ARGB 颜色；透明色不产生形状。'),('Tracers','从视线前方向此类存储绘制示踪线；实体额外绘制高度线。')]:result[key(name+'.'+field)+'.description']=({'Enabled':'顯示此類儲存。','Color':'此類儲存的 ARGB 色彩；透明色不產生形狀。','Tracers':'從視線前方向此類儲存繪製追蹤線；實體額外繪製高度線。'}[field]if language=='zh_tw'else help)
 for style,fields in zip(BASE['NAMES'],BASE['FIELDS']):
  for name in ['Opacity','Fill']+fields:
   text=(BASE['PARAMS_TW']if language=='zh_tw'else BASE['PARAMS'])[name]
   if name=='Fill':text='剪影内部填充不透明度；保留各原样式的独立默认值。'if language!='zh_tw'else'剪影內部填滿不透明度；保留各原樣式的獨立預設值。'
   text=text.replace('轨迹','存储').replace('軌跡','儲存');result[key(f'Mode.Glow.Style.{style}.{name}')+'.description']=text
 if language=='en_us':
  result={k:v for k,v in source.items()if k.startswith('liquidbounce.module.storageESP.')}
  for p in list(HELP)+[f'{n}.{f}'for n in TYPES for f in ['Enabled','Color','Tracers']]+[f'Mode.Glow.Style.{s}.{n}'for s,fields in zip(BASE['NAMES'],BASE['FIELDS'])for n in ['Opacity','Fill']+fields]:result.setdefault(key(p)+'.description',f'{p}: original StorageESP setting; model silhouettes, block shapes and tracers retain their source behavior.')
 return result
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(BASE['LABELS_TW']if language=='zh_tw'else BASE['LABELS']);data['labels'].update(LABELS_TW if language=='zh_tw'else LABELS);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,ROOT/a.directory/f'display_{lang}.json')
