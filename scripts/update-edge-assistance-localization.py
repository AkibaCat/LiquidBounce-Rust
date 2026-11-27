#!/usr/bin/env python3
"""Owned Parkour and SafeWalk simplified/traditional display vocabulary."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
TEXTS={
 'parkour.assist.enabled':('Plan gap crossings, time take-off, steer in air and settle on narrow landings.','规划跨越空隙的落点，选择起跳时机，修正空中方向并在狭窄落点停稳。','規劃跨越空隙的落點，選擇起跳時機，修正空中方向並在狹窄落點停穩。'),
 'parkour.assist.maxAngle':('Search for landings within this angle of movement heading. Default 35 degrees.','在移动方向两侧此角度内搜索落点，默认 35 度。','在移動方向兩側此角度內搜尋落點，預設 35 度。'),
 'parkour.assist.render.enabled':('Preview the jump trajectory, take-off, landing and aim point.','预览跳跃轨迹、起跳位置、落点和瞄准点。','預覽跳躍軌跡、起跳位置、落點和瞄準點。'),
 'parkour.assist.render.canBeCovered':('Allow blocks to occlude the jump preview.','允许方块遮挡跳跃预览。','允許方塊遮擋跳躍預覽。'),
 'parkour.assist.render.arc':('Color of the predicted movement arc.','预测移动轨迹的颜色。','預測移動軌跡的顏色。'),
 'parkour.assist.render.takeOff':('Color of the take-off line and run-up marker.','起跳线和助跑标记的颜色。','起跳線和助跑標記的顏色。'),
 'parkour.assist.render.landing':('Color of the landing block and touchdown footprint.','落脚方块和预计落地点轮廓的颜色。','落腳方塊和預計落地點輪廓的顏色。'),
 'parkour.assist.render.aim':('Color of the aiming post and cross at eye height.','瞄准立柱及视线高度十字的颜色。','瞄準立柱及視線高度十字的顏色。'),
 'safeWalk.mode':('Safe applies the vanilla sneaking edge restriction. OnEdge changes keys near a ledge; None does neither.','Safe 使用原版潜行的防跌落限制。OnEdge 在边缘修改按键；None 不施加限制。','Safe 使用原版潛行的防跌落限制。OnEdge 在邊緣修改按鍵；None 不施加限制。'),
 'safeWalk.mode.onEdge.distance':('Random edge check distance, capped at current horizontal speed. Default 0.1–0.15 blocks.','随机的边缘检测距离，不超过当前水平速度。默认 0.1–0.15 格。','隨機的邊緣偵測距離，不超過目前水平速度。預設 0.1–0.15 格。'),
 'safeWalk.mode.onEdge.keep':('Random number of ticks to keep changing movement after detecting an edge. Default 1–2.','发现边缘后持续修改移动的随机刻数，默认 1–2 刻。','發現邊緣後持續修改移動的隨機刻數，預設 1–2 刻。'),
 'safeWalk.mode.onEdge.mode':('Stop clears movement at low speed; Invert reverses keys; Center moves toward the last safe block center. High speed also uses Center.','Stop 在低速时停止移动；Invert 反转按键；Center 朝最后安全方块的中心移动。速度过快时也会使用 Center。','Stop 在低速時停止移動；Invert 反轉按鍵；Center 朝最後安全方塊的中心移動。速度過快時也會使用 Center。'),
 'safeWalk.mode.onEdge.sneak':('Random number of ticks to sneak after detecting an edge. Default 0.','发现边缘后潜行的随机刻数，默认 0。','發現邊緣後潛行的隨機刻數，預設 0。'),
 'safeWalk.mode.onEdge.jump':('Hold jump during the movement override. Disabled by default.','在修改移动期间按住跳跃，默认关闭。','在修改移動期間按住跳躍，預設關閉。'),
}
LABELS={'Assist':('辅助','輔助'),'MaxAngle':('最大角度','最大角度'),'Arc':('轨迹','軌跡'),'TakeOff':('起跳','起跳'),'Landing':('落点','落點'),'Aim':('瞄准','瞄準'),'OnEdge':('边缘动作','邊緣動作'),'Safe':('安全','安全'),'Keep':('保持时间','保持時間'),'Invert':('反转','反轉'),'Center':('回到中心','回到中心')}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
 data['moduleNames'].update({'Parkour':('自动跑酷','自動跑酷')[index-1],'SafeWalk':('防跌落','防跌落')[index-1]});data['labels'].update({k:v[index-1] for k,v in LABELS.items()})
 for name,text in TEXTS.items():
  key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=text[index]
  if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
 for field in ('descriptions','englishDescriptions'):
  for key,value in list(data.get(field,{}).items()):
   if key.startswith('liquidbounce.module.noRotateSet.mode.resetRotation.rotations'):
    data[field][key.replace('liquidbounce.module.noRotateSet.mode.resetRotation.rotations','liquidbounce.module.parkour.assist.rotations',1)]=value
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
