from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
for code,index in [('zh_cn',1),('zh_tw',2)]:
 p=HERE.parent/'liquidbounce-interop/resources/localization'/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['moduleNames']['AntiStaff']=['管理提醒','管理提醒'][index-1];d['labels']['ShowInTabList']=['在玩家列表标记','在玩家清單標記'][index-1]
 for key,text in {
 'showInTabList.description':('Mark known server staff in the tab player list.','在 Tab 玩家列表中标记名单内的服务器管理。','在 Tab 玩家清單中標記名單內的伺服器管理。'),
 'messages.staffDetected':('Staff Detected','检测到管理','偵測到管理'),
 'messages.staffSuffix':(' - (Staff)',' -（管理）',' -（管理）')}.items():
  path='liquidbounce.module.antiStaff.'+key;d['descriptions'][path]=text[index]
  if index==1:d['englishDescriptions'][path]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(code,'AntiStaff labels, help and runtime texts')
