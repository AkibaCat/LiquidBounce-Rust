"""Radar's complete live Simplified/Traditional Chinese labels and help."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Show rendered entities with directional arrows and name tags.','用方向箭头和名称显示已渲染实体的位置。','以方向箭頭及名稱顯示已繪製實體的位置。'),
 'onlyPlayers':('Only show player entities.','只显示玩家实体。','只顯示玩家實體。'),
 'hideTeammates':('Hide entities matching the Teams module criteria, regardless of whether Teams is enabled.','根据队伍模块设置隐藏队友，无需启用队伍模块。','根據隊伍模組設定隱藏隊友，無須啟用隊伍模組。'),
 'show':('Choose indicators inside the viewport, offscreen, or both.','选择显示视野内、视野外或两者的指示器。','選擇顯示視野內、視野外或兩者的指示器。'),
 'edgeInset':('Inset the indicator frame by this fraction of each screen dimension.','按屏幕尺寸的此比例将指示边界向内缩进。','按螢幕尺寸的此比例將指示邊界向內縮排。'),
 'pointerMode':('Choose a shaped triangle or one of the two original arrow images.','选择三角形或原版的两种箭头图片。','選擇三角形或原版的兩種箭頭圖片。'),
 'pointerMode.triangle':('Draw a configurable concave triangle.','绘制可调节凹尾的三角箭头。','繪製可調整凹尾的三角箭頭。'),
 'pointerMode.triangle.width':('Triangle width in GUI pixels.','三角箭头的界面像素宽度。','三角箭頭的介面像素寬度。'),
 'pointerMode.triangle.height':('Triangle length in GUI pixels.','三角箭头的界面像素长度。','三角箭頭的介面像素長度。'),
 'pointerMode.triangle.tailConcaveSize':('Tail indentation, limited to the current triangle height when assigned.','尾部凹陷深度；设置时不超过当前箭头高度。','尾部凹陷深度；設定時不超過目前箭頭高度。'),
 'pointerMode.image1':('Use the first original triangle texture.','使用原版第一种箭头纹理。','使用原版第一種箭頭材質。'),
 'pointerMode.image2':('Use the second original triangle texture.','使用原版第二种箭头纹理。','使用原版第二種箭頭材質。'),
 'pointerMode.image1.size':('First image arrow size in GUI pixels.','第一种图片箭头的界面像素大小。','第一種圖片箭頭的介面像素大小。'),
 'pointerMode.image2.size':('Second image arrow size in GUI pixels.','第二种图片箭头的界面像素大小。','第二種圖片箭頭的介面像素大小。'),
 'colorMode':('Choose distance, health, static, or rainbow color.','选择距离、生命值、固定或彩虹颜色。','選擇距離、生命值、固定或彩虹顏色。'),
 'colorMode.distance':('Color arrows by current entity distance from the camera.','根据实体当前位置与相机的距离设置颜色。','根據實體目前位置與相機的距離設定顏色。'),
 'colorMode.distance.saturation':('Distance color saturation.','距离颜色的饱和度。','距離顏色的飽和度。'),
 'colorMode.distance.brightness':('Distance color brightness.','距离颜色的亮度。','距離顏色的亮度。'),
 'colorMode.distance.hue':('Hue curve in degrees against distance.','距离对应的色相曲线，单位为度。','距離對應的色相曲線，單位為度。'),
 'colorMode.health':('Blend red to green according to actual health.','按实际生命值从红色过渡到绿色。','按實際生命值從紅色過渡至綠色。'),
 'colorMode.health.alpha':('Stored health color opacity; the module Alpha curve replaces it for indicators.','保留生命值颜色透明度；实际指示器使用模块的透明度曲线。','保留生命值顏色透明度；實際指示器使用模組的透明度曲線。'),
 'colorMode.static':('Use a fixed color.','使用固定颜色。','使用固定顏色。'),
 'colorMode.static.color':('Fixed arrow and default name color.','固定的箭头颜色和默认名称颜色。','固定的箭頭顏色與預設名稱顏色。'),
 'colorMode.rainbow':('Cycle through rainbow colors.','循环显示彩虹颜色。','循環顯示彩虹顏色。'),
 'alpha':('Opacity curve against interpolated camera distance.','按插值后的相机距离计算透明度。','按插值後的相機距離計算透明度。'),
}
LABELS={'Radar':('方向雷达','方向雷達'),'PointerMode':('箭头模式','箭頭模式'),'TailConcaveSize':('尾部凹陷深度','尾部凹陷深度'),'EdgeInset':('边缘缩进比例','邊緣縮排比例'),'OnlyPlayers':('仅显示玩家','僅顯示玩家'),'HideTeammates':('隐藏队友','隱藏隊友'),'InView':('视野内','視野內'),'Offscreen':('视野外','視野外'),'Image1':('图片一','圖片一'),'Image2':('图片二','圖片二')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  p=folder/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));suffix=lambda k:k if k=='description'else k+'.description'
  put_module(d,'liquidbounce.module.radar','Radar',{'':LABELS['Radar'][index-1],**{suffix(k):v[index]for k,v in TEXTS.items()}},{suffix(k):v[0]for k,v in TEXTS.items()});d['labels'].update({k:v[index-1]for k,v in LABELS.items()});p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(a.parse_args().directory)
