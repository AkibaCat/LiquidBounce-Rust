from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
'choice.fancy.horizontal':('Maximum horizontal search distance; a blocked position must occur before a reachable opening.','水平搜索的最大距离，必须先遇到障碍才能穿到另一侧。','水平搜尋的最大距離，必須先遇到障礙才能穿到另一側。'),
'choice.fancy.vertical':('Maximum upward and downward search distance.','向上和向下搜索的最大距离。','向上和向下搜尋的最大距離。'),
'choice.fancy.allowedLiquids':('Allow destinations in the selected fluids when the space above is air or the same fluid.','允许落点位于选中的液体内，上方必须为空气或同种液体。','允許落點位於選取的液體內，上方必須為空氣或同種液體。'),
'choice.fancy.requiresStandOn':('Require center support below the destination, except when clipping upward.','除向上穿越外，要求落点下方有可站立的中心支撑。','除向上穿越外，要求落點下方有可站立的中心支撐。'),
'choice.old.horizontal':('Horizontal offset along the facing direction when enabled.','启用时沿朝向移动的水平距离。','啟用時沿朝向移動的水平距離。'),
'choice.old.vertical':('Vertical offset when enabled; negative values move downward.','启用时移动的垂直距离，负数表示向下。','啟用時移動的垂直距離，負數表示向下。'),
'choice.old.resetVelocity':('Clear movement velocity after the one-time offset.','执行一次位置偏移后清除移动速度。','執行一次位置偏移後清除移動速度。')}
for lang,index in [('zh_cn',1),('zh_tw',2)]:
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['descriptions'].update({f'liquidbounce.module.clip.{k}.description':v[index] for k,v in HELP.items()})
 d['descriptions']['liquidbounce.module.clip.messages.whoosh']='嗖！' if index==1 else '咻！'
 d['labels'].update({'AllowedLiquids':'允许的液体' if index==1 else '允許的液體','RequiresStandOn':'需要落脚支撑' if index==1 else '需要落腳支撐','ResetVelocity':'重置速度','Fancy':'智能','Old':'传统' if index==1 else '傳統'})
 if index==1:
  d['englishDescriptions'].update({f'liquidbounce.module.clip.{k}.description':v[0] for k,v in HELP.items()})
  d['englishDescriptions']['liquidbounce.module.clip.messages.whoosh']='Whoosh!'
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Clip simplified/traditional Chinese')
