from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
SWITCHES=['JoinMessages','LeaveMessages','GameModeMessages','ItemConsumptionMessages','HeldItemMessages','TotemPopMessages']
FORMATS=['JoinMessageFormat','LeaveMessageFormat','GameModeMessageFormat','ItemConsumptionMessageFormat','HeldItemMessageFormat','TotemPopMessageFormat']
CN=['加入提醒','离开提醒','游戏模式提醒','物品使用提醒','手持物品提醒','图腾触发提醒']
TW=['加入提醒','離開提醒','遊戲模式提醒','物品使用提醒','手持物品提醒','圖騰觸發提醒']
EN=['join','leave','game mode changes','completed consumption','held item changes','totem pops']
TEXTS={
 'default0':('%s joined','%s 加入了游戏','%s 加入了遊戲'),
 'default1':('%s left','%s 离开了游戏','%s 離開了遊戲'),
 'default2':('%s changed their game mode to %s','%s 将游戏模式更改为 %s','%s 將遊戲模式變更為 %s'),
 'default3':('%1$s used %2$s','%1$s 使用了 %2$s','%1$s 使用了 %2$s'),
 'default4':('%1$s holds %2$s x%3$s in %4$s','%1$s 的%4$s拿着 %2$s ×%3$s','%1$s 的%4$s拿著 %2$s ×%3$s'),
 'default5':('%1$s popped a totem %2$s times','%1$s 已触发 %2$s 次图腾','%1$s 已觸發 %2$s 次圖騰'),
 'SURVIVAL':('SURVIVAL','生存模式','生存模式'),'CREATIVE':('CREATIVE','创造模式','創造模式'),
 'ADVENTURE':('ADVENTURE','冒险模式','冒險模式'),'SPECTATOR':('SPECTATOR','旁观模式','旁觀模式'),
 'MAIN_HAND':('MAIN_HAND','主手','主手'),'OFF_HAND':('OFF_HAND','副手','副手')}
for code,index in [('zh_cn',1),('zh_tw',2)]:
 p=HERE.parent/'liquidbounce-interop/resources/localization'/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['moduleNames']['Notifier']='事件提醒'
 for i,name in enumerate(SWITCHES):
  d['labels'][name]=(CN if index==1 else TW)[i]
  d['labels'][FORMATS[i]]=((CN if index==1 else TW)[i])+('格式')
  for n,text in [(name,(f'Notify about {EN[i]}.',f'显示{CN[i]}。',f'顯示{TW[i]}。')),(FORMATS[i],('Java-style message format. The unchanged default follows the game language.','消息格式，支持 Java 格式参数；默认格式随游戏语言切换。','訊息格式，支援 Java 格式參數；預設格式隨遊戲語言切換。'))]:
   key='liquidbounce.module.notifier.'+n[0].lower()+n[1:]+'.description';d['descriptions'][key]=text[index]
   if index==1:d['englishDescriptions'][key]=text[0]
 for name,text in [('HeldItems',('Items whose main-hand and off-hand count/component changes are reported.','报告这些物品在主副手的数量与组件变化。','回報這些物品在主副手的數量與元件變更。')),('UseNotification',('Show notifications instead of local chat.','以通知显示提醒。','以通知顯示提醒。'))]:
  d['labels'][name]=('跟踪物品'if index==1 else'追蹤物品')if name=='HeldItems'else'使用通知'
  key='liquidbounce.module.notifier.'+name[0].lower()+name[1:]+'.description';d['descriptions'][key]=text[index]
  if index==1:d['englishDescriptions'][key]=text[0]
 for key,text in TEXTS.items():
  key='liquidbounce.module.notifier.messages.'+key;d['descriptions'][key]=text[index]
  if index==1:d['englishDescriptions'][key]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(code,'Notifier labels/help/templates')
