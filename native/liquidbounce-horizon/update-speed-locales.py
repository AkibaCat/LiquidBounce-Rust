#!/usr/bin/env python3
"""Add complete ModuleSpeed labels/descriptions for zh_cn and zh_tw."""
from pathlib import Path
import argparse,json
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.speed'
CN={'Speed':'加速','Mode':'模式','LegitHop':'合法跳跃','Custom':'自定义','YPort':'Y轴端口','PiercingAttack':'穿刺攻击','Verus-b3882':'Verus-b3882','HypixelBHop':'Hypixel跳跃','HypixelLowHop':'Hypixel低跳','Spartan-4.0.4.3':'Spartan-4.0.4.3','Spartan-4.0.4.3-FastFall':'Spartan快速落地','CubeCraft-2024-06-30':'CubeCraft-2024-06-30','Vulcan-2.8.6':'Vulcan-2.8.6','Vulcan-2.8.8':'Vulcan-2.8.8','Vulcan-2.8.6-Ground':'Vulcan地面','Grim-2.3.73':'Grim-2.3.73','NCP-2024-08-31':'NCP-2024-08-31','Intave-14':'Intave-14','Intave-14-Fast':'Intave-14快速','HylexLowHop':'Hylex低跳','HylexGround':'Hylex地面','BlocksMC':'BlocksMC','Matrix-7':'Matrix-7','Rewound':'Rewound','Not':'排除条件','AvoidEdgeBump':'避免边缘碰撞','OnlyInCombat':'仅战斗中','OnlyOnPotionEffect':'仅有药水效果时','PotionEffect':'药水效果','Speed':'迅捷','Slowness':'缓慢','Both':'两者','YawOffset':'偏转视角','YawOffsetMode':'视角偏转模式','Ground':'地面','Air':'空中','Constant':'固定','Enabled':'启用','HorizontalModification':'水平修改','HorizontalAcceleration':'水平加速度','HorizontalJumpOff':'起跳水平倍率','TicksToBoostOff':'起跳加速延迟','VerticalModification':'垂直修改','JumpHeight':'跳跃高度','Pulldown':'下压','PullDownDuringFall':'下落时下压','TimerSpeed':'计时速度','Strafe':'侧移','Strength':'强度','CustomSpeed':'自定义速度','VelocityTimeout':'速度保持时间','StrafeKnock':'击退后侧移','Glide':'滑翔','VerticalAcceleration':'垂直加速','RoundStrafeYaw':'将侧移角度取整','LevelRange':'等级范围','PotionEffectsBoostRange':'药水效果倍率范围','Rotations':'旋转'}
TW={'Speed':'加速','Mode':'模式','LegitHop':'合法跳躍','Custom':'自訂','YPort':'Y軸端口','PiercingAttack':'穿刺攻擊','HypixelBHop':'Hypixel跳躍','HypixelLowHop':'Hypixel低跳','Not':'排除條件','AvoidEdgeBump':'避免邊緣碰撞','OnlyInCombat':'僅戰鬥中','OnlyOnPotionEffect':'僅有藥水效果時','PotionEffect':'藥水效果','Speed':'迅捷','Slowness':'緩慢','Both':'兩者','YawOffset':'偏轉視角','YawOffsetMode':'視角偏轉模式','Ground':'地面','Air':'空中','Constant':'固定','Enabled':'啟用','HorizontalModification':'水平修改','HorizontalAcceleration':'水平加速度','HorizontalJumpOff':'起跳水平倍率','TicksToBoostOff':'起跳加速延遲','VerticalModification':'垂直修改','JumpHeight':'跳躍高度','Pulldown':'下壓','PullDownDuringFall':'下落時下壓','TimerSpeed':'計時速度','Strafe':'側移','Strength':'強度','CustomSpeed':'自訂速度','VelocityTimeout':'速度保持時間','StrafeKnock':'擊退後側移','Glide':'滑翔','VerticalAcceleration':'垂直加速','RoundStrafeYaw':'將側移角度取整','LevelRange':'等級範圍','PotionEffectsBoostRange':'藥水效果倍率範圍','Rotations':'旋轉'}
def key(path):return PREFIX+'.'+'.'.join('fov' if p=='FOV' else p[:1].lower()+p[1:] for p in path.split('.'))
def update(folder):
 for code,labels in [('zh_cn',CN),('zh_tw',TW)]:
  p=folder/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'));d.setdefault('moduleNames',{})['Speed']=labels['Speed'];ls=d.setdefault('labels',{});desc=d.setdefault('descriptions',{})
  for name,val in labels.items():
   ls[key(name)]=val
   desc[key(name)+'.description']=f'ModuleSpeed 设置：{val}。'
  desc[PREFIX+'.description']='按原版 LiquidBounce 规则提供完整的跳跃、侧移、低跳与反作弊适配模式。'
  p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(ap.parse_args().folder)
