from pathlib import Path
import argparse,json
from display_locale_schema import put_module
CN={
'':'保护区域','description':'显示保护方块覆盖的区域，并提示按网格衔接的新放置位置。',
'protectionBlocks':'保护方块','protectionBlocks.description':'识别为保护区域中心的方块，默认为绿宝石块。',
'protectionRadius':'保护半径','protectionRadius.description':'保护方块沿各坐标轴覆盖的范围。',
'protectionRadius.radiusX':'X 轴半径','protectionRadius.radiusX.description':'中心方块沿 X 轴两侧的保护半径。',
'protectionRadius.radiusZ':'Z 轴半径','protectionRadius.radiusZ.description':'中心方块沿 Z 轴两侧的保护半径。',
'protectionRadius.radiusY':'Y 轴半径','protectionRadius.radiusY.description':'中心方块上下的保护半径，区域会裁剪到世界高度范围。',
'placementIndicator':'放置提示','placementIndicator.description':'手持保护方块时显示按最近中心对齐的下一处网格位置。',
'placementIndicator.snapToY':'对齐中心高度','placementIndicator.snapToY.description':'放置提示使用最近保护中心的高度；关闭时使用玩家脚部高度。',
'renderer':'渲染','renderer.description':'设置显示区域的数量、条件和颜色。',
'renderer.renderLimit':'区域数量上限','renderer.renderLimit.description':'仅显示水平方向距离最近的指定数量保护区域。',
'renderer.holdBlockToRender':'手持方块时显示','renderer.holdBlockToRender.description':'仅当主手或副手持有保护方块时渲染区域。',
'renderer.protectionColors':'区域颜色','renderer.protectionColors.description':'保护区域和中心方块的颜色。',
'renderer.protectionColors.zoneFill':'区域填充','renderer.protectionColors.zoneFill.description':'玩家位于区域外且距离边界不超过 5 格时的填充颜色。',
'renderer.protectionColors.zoneOutline':'区域边框','renderer.protectionColors.zoneOutline.description':'保护区域外边界的线条颜色。',
'renderer.protectionColors.centerZoneOutline':'中心方块边框','renderer.protectionColors.centerZoneOutline.description':'保护中心方块的线条颜色。',
'renderer.indicatorColors':'提示颜色','renderer.indicatorColors.description':'建议放置位置的填充和边框颜色。',
'renderer.indicatorColors.indicatorOutline':'提示边框','renderer.indicatorColors.indicatorOutline.description':'建议放置方块位置的线条颜色。',
'renderer.indicatorColors.indicatorFill':'提示填充','renderer.indicatorColors.indicatorFill.description':'建议放置方块位置的填充颜色。',
}
TW={
'':'保護區域','description':'顯示保護方塊覆蓋的區域，並提示按網格銜接的新放置位置。',
'protectionBlocks':'保護方塊','protectionBlocks.description':'識別為保護區域中心的方塊，預設為綠寶石方塊。',
'protectionRadius':'保護半徑','protectionRadius.description':'保護方塊沿各座標軸覆蓋的範圍。',
'protectionRadius.radiusX':'X 軸半徑','protectionRadius.radiusX.description':'中心方塊沿 X 軸兩側的保護半徑。',
'protectionRadius.radiusZ':'Z 軸半徑','protectionRadius.radiusZ.description':'中心方塊沿 Z 軸兩側的保護半徑。',
'protectionRadius.radiusY':'Y 軸半徑','protectionRadius.radiusY.description':'中心方塊上下的保護半徑，區域會裁剪至世界高度範圍。',
'placementIndicator':'放置提示','placementIndicator.description':'手持保護方塊時顯示按最近中心對齊的下一處網格位置。',
'placementIndicator.snapToY':'對齊中心高度','placementIndicator.snapToY.description':'放置提示使用最近保護中心的高度；關閉時使用玩家腳部高度。',
'renderer':'繪製','renderer.description':'設定顯示區域的數量、條件和顏色。',
'renderer.renderLimit':'區域數量上限','renderer.renderLimit.description':'僅顯示水平方向距離最近的指定數量保護區域。',
'renderer.holdBlockToRender':'手持方塊時顯示','renderer.holdBlockToRender.description':'僅當主手或副手持有保護方塊時繪製區域。',
'renderer.protectionColors':'區域顏色','renderer.protectionColors.description':'保護區域和中心方塊的顏色。',
'renderer.protectionColors.zoneFill':'區域填色','renderer.protectionColors.zoneFill.description':'玩家位於區域外且距離邊界不超過 5 格時的填色。',
'renderer.protectionColors.zoneOutline':'區域邊框','renderer.protectionColors.zoneOutline.description':'保護區域外邊界的線條顏色。',
'renderer.protectionColors.centerZoneOutline':'中心方塊邊框','renderer.protectionColors.centerZoneOutline.description':'保護中心方塊的線條顏色。',
'renderer.indicatorColors':'提示顏色','renderer.indicatorColors.description':'建議放置位置的填色和邊框顏色。',
'renderer.indicatorColors.indicatorOutline':'提示邊框','renderer.indicatorColors.indicatorOutline.description':'建議放置方塊位置的線條顏色。',
'renderer.indicatorColors.indicatorFill':'提示填色','renderer.indicatorColors.indicatorFill.description':'建議放置方塊位置的填色。',
}
EN={
 'description':'Display regions covered by protection blocks and suggest aligned placement positions.',
 'protectionBlocks.description':'Blocks treated as protection-zone centers; emerald block by default.',
 'protectionRadius.description':'Coverage of a protection block along each coordinate axis.',
 'protectionRadius.radiusX.description':'Protection radius on both sides of the center along the X axis.',
 'protectionRadius.radiusZ.description':'Protection radius on both sides of the center along the Z axis.',
 'protectionRadius.radiusY.description':'Protection radius above and below the center, clipped to world height.',
 'placementIndicator.description':'While holding a protection block, show the next grid position aligned to the nearest center.',
 'placementIndicator.snapToY.description':'Use the nearest protection center height for placement hints, or player foot height when disabled.',
 'renderer.description':'Set the number of rendered zones, rendering conditions and colors.',
 'renderer.renderLimit.description':'Render only this many horizontally nearest protection zones.',
 'renderer.holdBlockToRender.description':'Render zones only while either hand holds a protection block.',
 'renderer.protectionColors.description':'Colors for protection zones and their center blocks.',
 'renderer.protectionColors.zoneFill.description':'Fill color when outside the zone and within five blocks of its boundary.',
 'renderer.protectionColors.zoneOutline.description':'Line color for the outer boundary of a protection zone.',
 'renderer.protectionColors.centerZoneOutline.description':'Line color for a protection-center block.',
 'renderer.indicatorColors.description':'Fill and outline colors for suggested placement positions.',
 'renderer.indicatorColors.indicatorOutline.description':'Outline color for a suggested block position.',
 'renderer.indicatorColors.indicatorFill.description':'Fill color for a suggested block position.',
}
def update(language,path):
 data=json.loads(path.read_text(encoding='utf8'))
 put_module(data,'liquidbounce.module.protectionZones','ProtectionZones',CN if language=='zh_cn'else TW,EN)
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
