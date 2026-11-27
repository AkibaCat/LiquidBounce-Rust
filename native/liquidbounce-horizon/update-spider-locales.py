#!/usr/bin/env python3
"""Owned Spider setting help, merged without replacing other modules."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'en_us':{'spider.mode.vanilla.motion':'Vertical velocity while touching a wall in Vanilla mode.','spider.mode.polar-2025-03-29.jumpHeight':'Jump motion while touching a wall in Polar mode. Values above 0.42 replace the original jump motion.'},
 'zh_cn':{'spider.mode.vanilla.motion':'Vanilla 模式中接触墙面时的垂直速度。','spider.mode.polar-2025-03-29.jumpHeight':'Polar 模式中接触墙面时的起跳速度。高于 0.42 时替换原本的跳跃速度。'},
 'zh_tw':{'spider.mode.vanilla.motion':'Vanilla 模式中接觸牆面時的垂直速度。','spider.mode.polar-2025-03-29.jumpHeight':'Polar 模式中接觸牆面時的起跳速度。高於 0.42 時替換原本的跳躍速度。'},
}
for language in ('zh_cn','zh_tw'):
 target=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json';data=json.loads(target.read_text(encoding='utf8'))
 upstream=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 key='liquidbounce.module.spider.description';data['descriptions'][key]=upstream[key]
 data['descriptions'].update({f'liquidbounce.module.{k}.description':v for k,v in HELP[language].items()})
 if language=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'));data['englishDescriptions'][key]=english[key]
  data['englishDescriptions'].update({f'liquidbounce.module.{k}.description':v for k,v in HELP['en_us'].items()})
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Spider simplified and traditional Chinese help')
