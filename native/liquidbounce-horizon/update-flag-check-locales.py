from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'FlagCheck':('回退检测','回退偵測'),'ChatMessage':('聊天提示','聊天提示'),'InvalidAttributes':('异常属性','異常屬性'),'ResetFlags':('重置计数','重設計數'),'After':('间隔','間隔'),'NotInFirstPerson':('第一人称隐藏','第一人稱隱藏'),'Alive':('停留时间','停留時間')}
TEXTS={
'':('Alert on server setbacks, forced rotations and invalid player attributes.','检测服务器回退、强制转头和玩家异常属性。','偵測伺服器回退、強制轉頭與玩家異常屬性。'),
'chatMessage':('Show alerts in chat; a new alert replaces the previous message of the same type.','在聊天中提示；同类型的新提示会替换旧提示。','在聊天中提示；同類型的新提示會取代舊提示。'),
'notification':('Show an informational notification for each alert.','每次检测时显示信息通知。','每次偵測時顯示資訊通知。'),
'invalidAttributes':('Check invalid health and empty hunger on each game tick.','每刻检查异常生命值和耗尽的饥饿值。','每刻檢查異常生命值與耗盡的飢餓值。'),
'resetFlags':('Periodically clear the detected flag count.','定时清空检测计数。','定時清空偵測計數。'),
'resetFlags.enabled':('Enable periodic count reset.','启用定时计数重置。','啟用定時計數重設。'),
'resetFlags.after':('Seconds between count resets.','每次清空计数的间隔秒数。','每次清空計數的間隔秒數。'),
'render':('Draw the player wireframe at the position and rotation sent by a setback.','在服务器回退指定的位置和角度绘制玩家线框。','在伺服器回退指定的位置與角度繪製玩家線框。'),
'render.enabled':('Show the most recent setback position.','显示最近一次回退的位置。','顯示最近一次回退的位置。'),
'render.notInFirstPerson':('Hide the wireframe in first person.','在第一人称视角隐藏线框。','在第一人稱視角隱藏線框。'),
'render.alive':('Time before the wireframe begins to fade.','线框开始淡出前的停留时间。','線框開始淡出前的停留時間。'),
'render.fadeOut':('Easing curve for the wireframe fade.','线框淡出使用的缓动曲线。','線框淡出使用的緩動曲線。'),
'render.outTime':('Duration of the fade after the initial display time.','停留结束后淡出的持续时间。','停留結束後淡出的持續時間。'),
'render.color':('Player wireframe fill color and opacity.','玩家线框的填充颜色和透明度。','玩家線框的填滿顏色與透明度。'),
'render.outlineColor':('Player wireframe outline color and opacity.','玩家线框的轮廓颜色和透明度。','玩家線框的輪廓顏色與透明度。')}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 p=FOLDER/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));d['moduleNames']['FlagCheck']=LABELS['FlagCheck'][index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
 for suffix,text in TEXTS.items():
  key='liquidbounce.module.flagCheck'+('.'+suffix if suffix else '')+'.description';d['descriptions'][key]=text[index]
  if locale=='zh_cn':d['englishDescriptions'][key]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'FlagCheck help entries')
