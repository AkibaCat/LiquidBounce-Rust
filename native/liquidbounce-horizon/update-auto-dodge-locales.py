from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'AllowRotationChange':('允许改变视角','允許改變視角'),'AllowJump':('允许跳跃','允許跳躍'),'AllowTimer':('允许加速','允許加速'),'TimerSpeed':('游戏速度','遊戲速度'),'OpenInventory':('打开物品栏时','開啟物品欄時'),'UsingItem':('使用物品时','使用物品時'),'UsingScaffold':('自动搭路时','自動搭路時')}
TEXTS={
'':('Predict arrows and tridents, then steer away from their flight path.','预测箭矢和三叉戟轨迹，自动调整移动方向躲避。','預測箭矢與三叉戟軌跡，自動調整移動方向躲避。'),
'ignore':('Selected conditions do not prevent dodging. Unselected conditions pause the module.','勾选的情况不会阻止躲避；未勾选的情况发生时暂停躲避。','勾選的情況不會阻止躲避；未勾選的情況發生時暫停躲避。'),
'allowRotationChange.enabled':('Allow camera rotation when directional keys alone cannot evade in time.','仅调整按键来不及躲避时，允许改变视角朝向。','僅調整按鍵來不及躲避時，允許改變視角朝向。'),
'allowRotationChange.allowJump':('Queue a jump on the next movement input when it improves evasion.','跳跃有助于躲避且玩家着地时，在下一次移动输入中跳跃。','跳躍有助於躲避且玩家著地時，在下一次移動輸入中跳躍。'),
'allowTimer.enabled':('Temporarily increase game speed when the forecast requires faster movement.','预测需要更快移动时，临时提高游戏速度。','預測需要更快移動時，臨時提高遊戲速度。'),
'allowTimer.timerSpeed':('Requested game speed for urgent evasion.','紧急躲避时申请的游戏速度倍率。','緊急躲避時申請的遊戲速度倍率。'),
}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 p=FOLDER/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));d['moduleNames']['AutoDodge']=('自动躲避','自動躲避')[index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
 for suffix,text in TEXTS.items():
  key='liquidbounce.module.autoDodge'+('.'+suffix if suffix else '')+'.description';d['descriptions'][key]=text[index]
  if locale=='zh_cn':d['englishDescriptions'][key]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'AutoDodge help entries')
