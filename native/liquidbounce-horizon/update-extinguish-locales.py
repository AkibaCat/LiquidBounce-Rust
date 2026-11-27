"""Extinguish settings and full Rotations descriptions, following the game language."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.extinguish'
TEXTS={
 'description':('Automatically extinguish yourself with water and optionally retrieve it.','着火时自动用水灭火，并可在指定时间窗口内收回水。','著火時自動用水滅火，並可在指定時間範圍內收回水。'),
 'cooldown.description':('Seconds after a successful bucket interaction before another extinguishing action. Picking up water also advances this cooldown.','成功使用水桶后，下次灭火前等待的秒数；收水成功也会延长此冷却。','成功使用水桶後，下次滅火前等待的秒數；收水成功也會延長此冷卻。'),
 'notDuringCombat.description':('Pause extinguishing and pickup while CombatManager reports combat.','战斗管理器判定正在战斗时，暂停灭火和收水。','戰鬥管理器判定正在戰鬥時，暫停滅火和收水。'),
 'pickup.description':('Retrieve the most recently placed water using an empty bucket, before planning new extinguishing.','优先用空桶收回最近放置的水，然后才考虑再次灭火。','優先用空桶收回最近放置的水，然後才考慮再次滅火。'),
 'pickup.enabled.description':('Enable automatic retrieval within the configured time window.','启用指定时间窗口内的自动收水。','啟用指定時間範圍內的自動收水。'),
 'pickup.pickupSpan.description':('Minimum and maximum seconds since placement; eligibility is strictly after the minimum and entries expire strictly after the maximum.','放水后允许收水的最短和最长秒数；严格超过最短时间才可收水，严格超过最长时间才过期。','放水後允許收水的最短和最長秒數；嚴格超過最短時間才可收水，嚴格超過最長時間才過期。'),
}
LABELS={'Extinguish':('自动灭火','自動滅火'),'Pickup':('收回水','收回水'),'PickupSpan':('收水时间窗口','收水時間範圍')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['Extinguish'][index-1],**{k:v[index]for k,v in TEXTS.items()}}
  put_module(data,PREFIX,'Extinguish',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
   for key,value in list(data.get(field,{}).items()):
    if key.startswith(old):data[field][key.replace(old,PREFIX+'.rotations')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');args=a.parse_args();update(args.folder)
