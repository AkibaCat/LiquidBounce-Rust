#!/usr/bin/env python3
"""Merge FastUse help in Simplified/Traditional Chinese with English fallback."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'en_us':{'conditions':'Accelerate only while the selected restrictions are satisfied.','stopInput':'Stop directional movement while the use key is held.','packetType':'Movement packet format used to accelerate item use.','mode.immediate.delay':'Wait this many ticks before sending packets; cancel if use conditions cease.','mode.immediate.timer':'Temporary timer speed while waiting to accelerate use.','mode.immediate.speed':'Movement packets sent before releasing the used item.','mode.itemUseTime.consumeTime':'Minimum item use time in ticks before acceleration.','mode.itemUseTime.speed':'Movement packets sent after the minimum use time.','mode.crossbow.tickCooldown':'Perform the hand swap and crossbow use sequence on this tick interval.'},
 'zh_cn':{'conditions':'仅在满足所选限制条件时加速使用。','stopInput':'按住使用键时停止方向移动输入。','packetType':'用于加速物品使用的移动数据包格式。','mode.immediate.delay':'发送数据包前等待的刻数；使用条件失效时取消。','mode.immediate.timer':'等待加速使用期间临时采用的游戏速度。','mode.immediate.speed':'释放使用中的物品前发送的移动数据包数。','mode.itemUseTime.consumeTime':'加速前至少使用物品的游戏刻数。','mode.itemUseTime.speed':'达到最短使用时间后发送的移动数据包数。','mode.crossbow.tickCooldown':'每隔这些游戏刻执行一次换手及弩使用操作。'},
 'zh_tw':{'conditions':'僅在滿足所選限制條件時加速使用。','stopInput':'按住使用鍵時停止方向移動輸入。','packetType':'用於加速物品使用的移動封包格式。','mode.immediate.delay':'傳送封包前等待的刻數；使用條件失效時取消。','mode.immediate.timer':'等待加速使用期間暫時採用的遊戲速度。','mode.immediate.speed':'釋放使用中的物品前傳送的移動封包數。','mode.itemUseTime.consumeTime':'加速前至少使用物品的遊戲刻數。','mode.itemUseTime.speed':'達到最短使用時間後傳送的移動封包數。','mode.crossbow.tickCooldown':'每隔這些遊戲刻執行一次換手及弩使用操作。'},
}
for lang in ('zh_cn','zh_tw'):
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 original=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{lang}.json').read_text(encoding='utf8'))
 key='liquidbounce.module.fastUse.description';d['descriptions'][key]=original[key]
 d['descriptions'].update({f'liquidbounce.module.fastUse.{k}.description':v for k,v in HELP[lang].items()})
 d['labels'].update({'ItemUseTime':'按使用时间' if lang=='zh_cn' else '依使用時間','ConsumeTime':'使用时间' if lang=='zh_cn' else '使用時間','TickCooldown':'触发间隔' if lang=='zh_cn' else '觸發間隔','StopInput':'停止移动输入' if lang=='zh_cn' else '停止移動輸入'})
 if lang=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  d['englishDescriptions'][key]=english[key]
  d['englishDescriptions'].update({f'liquidbounce.module.fastUse.{k}.description':v for k,v in HELP['en_us'].items()})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged FastUse simplified/traditional Chinese')
