#!/usr/bin/env python3
"""NoWeb simplified/traditional display vocabulary; retain machine identifiers."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'PlaceWater':('放水脱网','放水脫網'),'BreakOnWorld':('本地移除蜘蛛网','本機移除蜘蛛網'),'OnlyOnGround':('仅在地面','僅在地面'),'MotionY':('垂直速度','垂直速度'),'MotionYStrength':('垂直速度值','垂直速度值'),'Pickup':('回收水源','回收水源'),'PickupSpan':('回收时间范围','回收時間範圍')}
TEXTS={
 'noWeb':('Disables web slowdown.','处理蜘蛛网减速，或自动放水脱网并回收水源。','處理蜘蛛網減速，或自動放水脫網並回收水源。'),
 'noWeb.mode':('Air cancels web slowdown; Grim breaks web collision; Intave jumps and redirects movement; PlaceWater uses water buckets; Strafe sets movement speed.','选择空气、Grim、Intave、放水脱网或移动加速模式。','選擇空氣、Grim、Intave、放水脫網或移動加速模式。'),
 'noWeb.mode.grim-2.3.65.breakOnWorld':('Remove collided cobwebs from the client world as well as sending the original start, abort and stop digging packets.','在发送开始、取消、停止挖掘数据包的同时，从本地世界移除碰到的蜘蛛网。','在傳送開始、取消、停止挖掘封包的同時，從本機世界移除碰到的蜘蛛網。'),
 'noWeb.mode.strafe.strength':('Horizontal movement speed while moving in a web. Default 0.23.','在蜘蛛网内移动时的水平速度，默认 0.23。','在蜘蛛網內移動時的水平速度，預設 0.23。'),
 'noWeb.mode.strafe.onlyOnGround':('Only change horizontal movement while on the ground.','仅在着地时修改水平移动速度。','僅在著地時修改水平移動速度。'),
 'noWeb.mode.strafe.motionY':('Override vertical velocity while moving in a web.','在蜘蛛网内移动时覆盖垂直速度。','在蜘蛛網內移動時覆寫垂直速度。'),
 'noWeb.mode.strafe.motionY.enabled':('Enable vertical velocity override.','启用垂直速度修改。','啟用垂直速度修改。'),
 'noWeb.mode.strafe.motionY.motionYStrength':('Vertical velocity while moving in a web. Default 0.6.','在蜘蛛网内移动时的垂直速度，默认 0.6。','在蜘蛛網內移動時的垂直速度，預設 0.6。'),
 'noWeb.mode.placeWater.pickup':('Use an empty bucket to recover tracked source water after the configured delay.','按设定延迟使用空桶回收跟踪的水源。','按設定延遲使用空桶回收追蹤的水源。'),
 'noWeb.mode.placeWater.pickup.enabled':('Enable automatic water source pickup.','自动回收放置的水源。','自動回收放置的水源。'),
 'noWeb.mode.placeWater.pickup.pickupSpan':('Minimum and maximum time before picking up source water, in seconds. Default 0.8..3.0; minimum 0.5 seconds.','回收水源的最短等待和最长保留时间，单位秒；默认 0.8..3.0，最短不得低于 0.5 秒。','回收水源的最短等待和最長保留時間，單位秒；預設 0.8..3.0，最短不得低於 0.5 秒。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['NoWeb']=('蜘蛛网脱困','蜘蛛網脫困')[index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations';new='liquidbounce.module.noWeb.mode.placeWater.rotations'
  for bucket in ['descriptions']+(['englishDescriptions']if locale=='zh_cn'else[]):
   data[bucket].update({k.replace(old,new):v for k,v in list(data[bucket].items())if k.startswith(old)})
  for name,text in TEXTS.items():
   key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=text[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
  for suffix,text in {'compatibilityTitle':('Compatibility error','兼容性错误','相容性錯誤'),'compatibilityError':('NoWeb is incompatible with AvoidHazards','蜘蛛网脱困与危险规避的蜘蛛网选项不兼容，已关闭危险规避。','蜘蛛網脫困與危險規避的蜘蛛網選項不相容，已關閉危險規避。')}.items():
   key='liquidbounce.module.noWeb.messages.'+suffix;data['descriptions'][key]=text[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
