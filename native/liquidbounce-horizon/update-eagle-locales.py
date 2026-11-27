from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'edgeDistance':('Random distance at which to begin sneaking near an edge; refreshed when sneaking ends.','接近边缘时开始潜行的随机距离，停止潜行后重新抽取。','接近邊緣時開始潛行的隨機距離，停止潛行後重新抽取。'),
 'conditional':('Require all selected movement, item and pitch conditions.','要求同时满足所有选中的移动、持有物品和俯仰角条件。','要求同時符合所有選中的移動、手持物品與俯仰角條件。'),
 'conditional.conditions':('All selected conditions must pass. Sneak can take control while the physical sneak key is held.','所有选中的条件必须同时满足；选择潜行后，可在按住潜行键时控制边缘潜行。','所有選中的條件必須同時符合；選擇潛行後，可在按住潛行鍵時控制邊緣潛行。'),
 'conditional.pitch':('Allowed player pitch range, clamped to -90 through 90 degrees.','允许的玩家俯仰角范围，按 -90 至 90 度判断。','允許的玩家俯仰角範圍，依 -90 至 90 度判斷。'),
}
for lang,i in [('zh_cn',1),('zh_tw',2)]:
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['descriptions'].update({f'liquidbounce.module.eagle.{k}.description':v[i] for k,v in HELP.items()})
 if i==1:d['englishDescriptions'].update({f'liquidbounce.module.eagle.{k}.description':v[0] for k,v in HELP.items()})
 d['labels'].update({'Conditional':'条件控制'if i==1 else'條件控制','HoldingBlocks':'手持方块'if i==1 else'手持方塊','Conditions':'条件'if i==1 else'條件','EdgeDistance':'边缘距离'if i==1 else'邊緣距離'})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Eagle simplified/traditional Chinese')
