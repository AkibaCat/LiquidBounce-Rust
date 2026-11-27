from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'filter':('Choose whether listed items are dropped or kept.','选择丢弃列表中的物品或保留列表中的物品。','選擇丟棄清單中的物品或保留清單中的物品。'),
 'items':('Items selected by the item filter.','物品过滤器使用的物品列表。','物品篩選器使用的物品清單。'),
 'autoClose':('Close the captured container after all selected stacks are dropped.','丢弃所有选中物品堆叠后关闭该容器。','丟棄所有選中物品堆疊後關閉該容器。'),
 'checkScreenHandlerType':('Restrict cleaning by actual container menu type.','按实际容器菜单类型限制清理。','依實際容器選單類型限制清理。'),
 'checkScreenHandlerType.types':('Container menu types accepted by the type filter.','类型过滤器使用的容器菜单类型。','類型篩選器使用的容器選單類型。'),
 'checkScreenHandlerType.filter':('Allow or exclude the selected menu types.','允许或排除选中的菜单类型。','允許或排除選中的選單類型。'),
 'checkScreenTitle':('Restrict cleaning by the visible container title.','按容器显示标题限制清理。','依容器顯示標題限制清理。'),
 'checkScreenTitle.titles':('Match these standard titles in the current game language.','匹配当前游戏语言中的这些标准标题。','比對目前遊戲語言中的這些標準標題。'),
 'checkScreenTitle.custom':('Additional titles matched exactly, including capitalization.','额外精确匹配的标题，区分大小写。','額外精確比對的標題，區分大小寫。'),
 'checkScreenTitle.filter':('Allow or exclude matching standard and custom titles.','允许或排除匹配的标准和自定义标题。','允許或排除相符的標準與自訂標題。'),
}
for lang,index in [('zh_cn',1),('zh_tw',2)]:
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 for category in ['descriptions','englishDescriptions']:
  if category in d:d[category].update({k.replace('elytraSwap','chestCleaner'):v for k,v in list(d[category].items()) if k.startswith('liquidbounce.module.elytraSwap.constraints.')})
 d['descriptions'].update({f'liquidbounce.module.chestCleaner.{k}.description':v[index] for k,v in HELP.items()})
 if index==1:d['englishDescriptions'].update({f'liquidbounce.module.chestCleaner.{k}.description':v[0] for k,v in HELP.items()})
 d['labels'].update({'CheckScreenHandlerType':'检查菜单类型' if index==1 else '檢查選單類型','CheckScreenTitle':'检查标题' if index==1 else '檢查標題','Types':'类型' if index==1 else '類型','Titles':'标题列表' if index==1 else '標題清單'})
 d['descriptions']['liquidbounce.module.chestCleaner.constraints.clickDelay.description']='相邻丢弃操作之间随机等待的毫秒数。' if index==1 else '相鄰丟棄操作之間隨機等待的毫秒數。'
 d['descriptions']['liquidbounce.module.chestCleaner.constraints.missChance.description']='每次丢弃操作开始时，随机模拟误点附近空槽位的概率。' if index==1 else '每次丟棄操作開始時，隨機模擬誤點附近空欄位的機率。'
 if index==1:
  d['englishDescriptions']['liquidbounce.module.chestCleaner.constraints.clickDelay.description']='Random delay in milliseconds between successive throw actions.'
  d['englishDescriptions']['liquidbounce.module.chestCleaner.constraints.missChance.description']='Chance of clicking a nearby empty container slot before a throw action.'
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged ChestCleaner simplified/traditional Chinese')
