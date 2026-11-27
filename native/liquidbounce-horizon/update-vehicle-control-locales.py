#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
texts={
 'baseSpeed':('基础速度','基礎速度'),'baseSpeed.horizontal':('水平速度','水平速度'),'baseSpeed.vertical':('垂直速度','垂直速度'),
 'sprintSpeed':('疾跑速度','疾跑速度'),'sprintSpeed.enabled':('启用疾跑速度','啟用疾跑速度'),'sprintSpeed.horizontal':('疾跑水平速度','疾跑水平速度'),'sprintSpeed.vertical':('疾跑垂直速度','疾跑垂直速度'),
 'glide':('自然滑落速度','自然滑落速度'),'mouseControl':('鼠标控制朝向','滑鼠控制朝向'),'noGlideOnSprint':('疾跑时停止滑落','疾跑時停止滑落'),
 'rehook':('重新骑乘','重新騎乘'),'rehook.enabled':('启用重新骑乘','啟用重新騎乘'),'rehook.unhookAfter':('下车间隔','下車間隔'),'rehook.hookAfter':('重新上车延迟','重新上車延遲'),
 'messages.quitHelp':('要从载具上离开，降落后按下潜行键。','要從載具上離開，降落後按下潛行鍵。'),
 'messages.vehicleGone':('载具消失了。','載具消失了。'),'messages.vehicleTooFar':('载具太远，无法触及。','載具太遠，無法觸及。'),
}
english={
 'baseSpeed':'Vehicle speed while not sprinting.',
 'baseSpeed.horizontal':'Horizontal vehicle speed while not sprinting.',
 'baseSpeed.vertical':'Vertical vehicle speed while jumping or sneaking.',
 'sprintSpeed':'Separate vehicle speed while holding sprint.',
 'sprintSpeed.enabled':'Use separate horizontal and vertical speed while holding sprint.',
 'sprintSpeed.horizontal':'Horizontal vehicle speed while holding sprint.',
 'sprintSpeed.vertical':'Vertical vehicle speed while holding sprint and jumping or sneaking.',
 'glide':'Downward drift when neither jumping nor sneaking; boats in water do not drift.',
 'mouseControl':'Turn the controlled vehicle with the player camera.',
 'noGlideOnSprint':'Stop downward drift while the sprint key is held.',
 'rehook':'Periodically detach and re-enter the stored vehicle.',
 'rehook.enabled':'Periodically detach and re-enter the stored vehicle.',
 'rehook.unhookAfter':'Ticks to remain aboard before detaching.',
 'rehook.hookAfter':'Ticks to wait before interacting with and re-entering the stored vehicle.',
}
for i,lang in enumerate(('zh_cn','zh_tw')):
 p=ROOT/'native/liquidbounce-interop/resources/localization'/f'{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 for k,v in texts.items():
  if k.startswith('messages.'):d['liquidbounce.module.vehicleControl.'+k]=v[i]
  else:d.pop('liquidbounce.module.vehicleControl.'+k,None)
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 display=ROOT/'native/liquidbounce-interop/resources/localization'/f'display_{lang}.json';ui=json.loads(display.read_text(encoding='utf8'))
 ui['moduleNames']['VehicleControl']=('载具控制','載具控制')[i]
 for name,k in [('BaseSpeed','baseSpeed'),('SprintSpeed','sprintSpeed'),('Glide','glide'),('MouseControl','mouseControl'),('NoGlideOnSprint','noGlideOnSprint'),('Rehook','rehook'),('UnhookAfter','rehook.unhookAfter'),('HookAfter','rehook.hookAfter')]:ui['labels'][name]=texts[k][i]
 for k,v in texts.items():
  if not k.startswith('messages.'):
   key='liquidbounce.module.vehicleControl.'+k+'.description';ui['descriptions'][key]=v[i]
   if lang=='zh_cn':ui['englishDescriptions'][key]=english[k]
 display.write_text(json.dumps(ui,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 print(lang,len(texts),'VehicleControl entries')
