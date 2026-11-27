#!/usr/bin/env python3
"""Owned setting help for NameCollector, Kick and AutoLeave."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'en_us':{
  'kick.mode':'Method used to leave a multiplayer server: disconnect, invalid position, self attack, invalid chat, invalid selected slot, or self interaction.',
  'autoLeave.health':'Leave when health is below this value. Creative mode, a single-player server, and a death-protection component in either hand interrupt the waiting condition.',
  'autoLeave.delay':'Random number of game ticks to wait. A zero delay checks the condition immediately. Positive delays check the condition on each following tick.',
  'autoLeave.mode':'Select one of the same six disconnect methods as Kick.',
 },
 'zh_cn':{
  'kick.mode':'选择离开多人服务器的方式：正常断开、无效坐标、攻击自己、非法聊天、非法快捷栏槽位或与自己交互。',
  'autoLeave.health':'生命值低于此数值时离开服务器。创造模式、单人游戏、任一手持物品带有死亡保护组件时会中断等待条件。',
  'autoLeave.delay':'随机等待的游戏 tick 数。0 表示立即判断；正数表示在后续每个 tick 检查是否中断等待。',
  'autoLeave.mode':'选择与 Kick 相同的六种离开服务器方式。',
 },
 'zh_tw':{
  'kick.mode':'選擇離開多人伺服器的方式：正常中斷、無效座標、攻擊自己、非法聊天、非法快捷列欄位或與自己互動。',
  'autoLeave.health':'生命值低於此數值時離開伺服器。創造模式、單人遊戲、任一手持物品帶有死亡保護元件時會中斷等待條件。',
  'autoLeave.delay':'隨機等待的遊戲 tick 數。0 表示立即判斷；正數表示在後續每個 tick 檢查是否中斷等待。',
  'autoLeave.mode':'選擇與 Kick 相同的六種離開伺服器方式。',
 },
}
for language in ('zh_cn','zh_tw'):
 target=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json'
 data=json.loads(target.read_text(encoding='utf8'))
 upstream=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 for module in ('nameCollector','kick','autoLeave'):
  key=f'liquidbounce.module.{module}.description'
  data['descriptions'][key]=upstream[key]
 data['descriptions'].update({f'liquidbounce.module.{key}.description':value for key,value in HELP[language].items()})
 labels={'Quit':'正常断开' if language=='zh_cn' else '正常中斷','InvalidPacket':'无效数据包' if language=='zh_cn' else '無效封包','SelfHurt':'攻击自己' if language=='zh_cn' else '攻擊自己','IllegalChat':'非法聊天','IllegalSwitchitem':'非法快捷栏槽位' if language=='zh_cn' else '非法快捷列欄位','IllegalInteract':'非法交互' if language=='zh_cn' else '非法互動'}
 for key,value in labels.items(): data['labels'].setdefault(key,value)
 if language=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  for module in ('nameCollector','kick','autoLeave'):
   key=f'liquidbounce.module.{module}.description';data['englishDescriptions'][key]=english[key]
  data['englishDescriptions'].update({f'liquidbounce.module.{key}.description':value for key,value in HELP['en_us'].items()})
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged NameCollector/Kick/AutoLeave simplified and traditional Chinese help')
