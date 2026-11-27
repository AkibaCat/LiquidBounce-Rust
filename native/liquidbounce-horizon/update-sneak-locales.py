from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
'mode.legit.onMagmaBlocksOnly':('Sneak only when a two-tick prediction touches magma and the player is not immune to its damage.','仅当两刻预测会触及岩浆块且玩家不免疫其伤害时潜行。','僅當兩刻預測會觸及岩漿塊且玩家不免疫其傷害時潛行。'),
'notDuringMove':('Do not force sneaking while movement input is nonzero.','存在非零移动输入时不强制潜行。','存在非零移動輸入時不強制潛行。')}
for language,index in [('zh_cn',1),('zh_tw',2)]:
 path=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json';d=json.loads(path.read_text(encoding='utf8'))
 d['descriptions'].update({f'liquidbounce.module.sneak.{k}.description':v[index] for k,v in HELP.items()})
 d['labels'].update({'OnMagmaBlocksOnly':'仅在岩浆块上' if index==1 else '僅在岩漿塊上','NotDuringMove':'移动时停用' if index==1 else '移動時停用'})
 if index==1:d['englishDescriptions'].update({f'liquidbounce.module.sneak.{k}.description':v[0] for k,v in HELP.items()})
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Sneak simplified/traditional Chinese')
