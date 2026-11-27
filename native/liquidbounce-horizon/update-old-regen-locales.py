#!/usr/bin/env python3
"""Owned OldRegen help for the existing English/Simplified/Traditional UI."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'en_us':{
  'health':'Send extra movement packets to accelerate legacy server regeneration.',
  'health.enabled':'Enable health based regeneration packets.',
  'health.health':'Target health in health points. Default 16; UI range 0–20.',
  'health.increment':'Packets per missing health point. Default 20; UI range 0–60.',
  'badEffects':'Accelerate the longest finite harmful effect using extra movement packets.',
  'fire':'Send at least nine packets while burning outside water and rain.',
  'maximumSpeed':'Cap only the harmful effect contribution. Health based packets retain the original independent count.',
  'timer':'Request the shared client timer while sending packets. Default 1.0; UI range 0.1–10.',
  'conditions':'Skip regeneration in selected situations: airborne, moving, regeneration effect active, or food below 20.',
  'packetType':'Choose status, position, rotation, or full movement packets.'},
 'zh_cn':{
  'health':'发送额外移动包，加速旧版服务器的生命恢复。',
  'health.enabled':'启用根据生命值计算的恢复包。',
  'health.health':'目标生命值，默认 16，界面范围 0～20。',
  'health.increment':'每缺少一点生命值发送的包数，默认 20，界面范围 0～60。',
  'badEffects':'根据持续时间最长的有限负面效果发送额外移动包。',
  'fire':'着火且不在水中或雨中时，每次至少发送 9 个包。',
  'maximumSpeed':'只限制负面效果贡献的包数，生命恢复包数保持原版独立计算。',
  'timer':'发送包时请求共享客户端计时器，默认 1.0，界面范围 0.1～10。',
  'conditions':'在选定情况暂停：离地、移动、拥有生命恢复效果，或饥饿值低于 20。',
  'packetType':'选择仅状态、位置与状态、视角与状态或完整移动包。'},
 'zh_tw':{
  'health':'傳送額外移動封包，加速舊版伺服器的生命恢復。',
  'health.enabled':'啟用依生命值計算的恢復封包。',
  'health.health':'目標生命值，預設 16，介面範圍 0～20。',
  'health.increment':'每缺少一點生命值傳送的封包數，預設 20，介面範圍 0～60。',
  'badEffects':'依持續時間最長的有限負面效果傳送額外移動封包。',
  'fire':'著火且不在水中或雨中時，每次至少傳送 9 個封包。',
  'maximumSpeed':'只限制負面效果貢獻的封包數，生命恢復封包數保持原版獨立計算。',
  'timer':'傳送封包時請求共享客戶端計時器，預設 1.0，介面範圍 0.1～10。',
  'conditions':'在選定情況暫停：離地、移動、擁有生命恢復效果，或飢餓值低於 20。',
  'packetType':'選擇僅狀態、位置與狀態、視角與狀態或完整移動封包。'}
}
LABELS={'zh_cn':{'Increment':'增量','BadEffects':'负面效果','MaximumSpeed':'最大速度','NotInTheAir':'离地时暂停','NotDuringMove':'移动时暂停','NotDuringRegeneration':'恢复效果期间暂停','DoNotCauseHunger':'避免饥饿','PacketType':'数据包类型','OnGroundOnly':'仅状态','PositionAndOnGround':'位置与状态','LookAndOnGround':'视角与状态'},'zh_tw':{'Increment':'增量','BadEffects':'負面效果','MaximumSpeed':'最大速度','NotInTheAir':'離地時暫停','NotDuringMove':'移動時暫停','NotDuringRegeneration':'恢復效果期間暫停','DoNotCauseHunger':'避免飢餓','PacketType':'封包類型','OnGroundOnly':'僅狀態','PositionAndOnGround':'位置與狀態','LookAndOnGround':'視角與狀態'}}
for language in ('zh_cn','zh_tw'):
 path=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json'
 data=json.loads(path.read_text(encoding='utf8'))
 for key,value in HELP[language].items():data['descriptions'][f'liquidbounce.module.oldRegen.{key}.description']=value
 for key,value in LABELS[language].items():data['labels'].setdefault(key,value)
 source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 data['descriptions']['liquidbounce.module.oldRegen.description']=source['liquidbounce.module.oldRegen.description']
 if language=='zh_cn':
  for key,value in HELP['en_us'].items():data['englishDescriptions'][f'liquidbounce.module.oldRegen.{key}.description']=value
  source=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  data['englishDescriptions']['liquidbounce.module.oldRegen.description']=source['liquidbounce.module.oldRegen.description']
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged OldRegen English, Simplified and Traditional help')
