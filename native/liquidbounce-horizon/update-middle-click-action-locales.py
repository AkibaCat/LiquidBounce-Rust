#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
'en_us':{'friendClicker.pickUpRange':'Maximum player ray distance for adding or removing friends.','pearl.slotResetDelay':'Ticks before the pearl hotbar selection resets.','pearl.stopOnSubmit':'Suppress pearl use while player pitch lies in this inclusive range.'},
'zh_cn':{'friendClicker.pickUpRange':'添加或移除好友时，玩家实体射线的最大距离。','pearl.slotResetDelay':'珍珠快捷栏选择恢复前等待的游戏刻数。','pearl.stopOnSubmit':'玩家俯仰角处于此范围内时停止投掷珍珠，包含两端。'},
'zh_tw':{'friendClicker.pickUpRange':'新增或移除好友時，玩家實體射線的最大距離。','pearl.slotResetDelay':'珍珠快捷欄選擇恢復前等待的遊戲刻數。','pearl.stopOnSubmit':'玩家俯仰角處於此範圍內時停止投擲珍珠，包含兩端。'},
}
for language in ('zh_cn','zh_tw'):
 target=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json';data=json.loads(target.read_text(encoding='utf8'))
 original=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'));key='liquidbounce.module.middleClickAction.description';data['descriptions'][key]=original[key]
 data['descriptions'].update({f'liquidbounce.module.middleClickAction.mode.{k}.description':v for k,v in HELP[language].items()})
 if language=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'));data['englishDescriptions'][key]=english[key]
  data['englishDescriptions'].update({f'liquidbounce.module.middleClickAction.mode.{k}.description':v for k,v in HELP['en_us'].items()})
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged MiddleClickAction Chinese help')
