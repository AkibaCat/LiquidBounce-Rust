#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={'en_us':{'matches':'Identify teammates by scoreboard alliance, the first visible name color, or a shared name prefix.','armorColor':'Also identify players with the same dyed color in any selected armor slot. Undyed items never match each other.','colorSources':'Resolve tag color from the first available source, respecting the stored source order.'},'zh_cn':{'matches':'通过计分板队伍、名称首个有效颜色或相同名称前缀识别队友。','armorColor':'所选护甲槽位中任一染色颜色相同的玩家也视为队友；未染色装备不会相互匹配。','colorSources':'按保存的来源顺序，采用首个可用的实体标记颜色。'},'zh_tw':{'matches':'透過計分板隊伍、名稱首個有效色彩或相同名稱前綴識別隊友。','armorColor':'所選護甲欄位中任一染色色彩相同的玩家也視為隊友；未染色裝備不會互相比對。','colorSources':'依儲存的來源順序，採用首個可用的實體標記色彩。'}}
for lang in ('zh_cn','zh_tw'):
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'));k='liquidbounce.module.teams.description'
 original=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{lang}.json').read_text(encoding='utf8'));d['descriptions'][k]=original[k]
 d['descriptions'].update({f'liquidbounce.module.teams.{k}.description':v for k,v in HELP[lang].items()})
 if lang=='zh_cn':
  english=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'));d['englishDescriptions'][k]=english[k]
  d['englishDescriptions'].update({f'liquidbounce.module.teams.{k}.description':v for k,v in HELP['en_us'].items()})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Teams simplified/traditional Chinese')
