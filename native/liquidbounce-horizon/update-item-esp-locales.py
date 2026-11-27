"""All ItemESP settings and shared glow styles in simplified/traditional Chinese."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Highlight dropped items and collectible projectiles through walls.','透视显示掉落物品及可拾取的投射物。','透視顯示掉落物品與可拾取的投射物。'),
 'filter':('Choose blacklist or whitelist for dropped items.','选择掉落物品的黑名单或白名单。','選擇掉落物品的黑名單或白名單。'),
 'items':('Item registry set used by the filter.','物品筛选列表。','物品篩選清單。'),
 'maximumDistance':('Maximum eye-to-camera distance in blocks.','实体眼睛与相机之间的最大距离，单位为格。','實體眼睛與相機之間的最大距離，單位為格。'),
 'tracers':('Draw lines from the camera direction to matching entities.','绘制从相机方向到符合条件实体的连线。','繪製從相機方向到符合條件實體的連線。'),
 'showArrows':('Include arrows allowed to be picked up.','显示允许拾取的箭。','顯示允許拾取的箭。'),
 'showArrows.enabled':('Enable arrow display.','启用箭的显示。','啟用箭的顯示。'),
 'showArrows.regularArrows':('Include regular arrows without potion color.','显示没有药水颜色的普通箭。','顯示沒有藥水顏色的一般箭。'),
 'showArrows.spectralArrows':('Include spectral arrows.','显示光灵箭。','顯示追跡之箭。'),
 'showArrows.arrowsWithEffects':('Include arrows with potion color.','显示带药水颜色的箭。','顯示帶藥水顏色的箭。'),
 'showTridents':('Include thrown tridents.','显示投出的三叉戟。','顯示擲出的三叉戟。'),
 'mode':('Choose glow, fixed item boxes or legacy 2D markers.','选择描边发光、固定物品方框或经典二维标记。','選擇描邊發光、固定物品方框或經典二維標記。'),
 'mode.glow':('Style the actual rendered model silhouettes.','对实际物品模型轮廓应用样式。','對實際物品模型輪廓套用樣式。'),
 'mode.glow.style':('Choose the original outline/glow style.','选择原版描边与发光样式。','選擇原版描邊與發光樣式。'),
 'mode.box':('Draw the original quarter-block item boxes.','绘制原版四分之一格物品方框。','繪製原版四分之一格物品方框。'),
 'mode.box.mergeIntersecting':('Merge transitively intersecting original boxes; touching faces stay separate.','合并相互穿插连通的方框；只接触边界的方框保持独立。','合併相互交疊連通的方框；僅接觸邊界的方框保持獨立。'),
 'mode.legacy2D':('Draw the original billboard corner markers.','绘制原版朝向相机的边角标记。','繪製原版朝向相機的邊角標記。'),
 'mode.legacy2D.scale':('Marker scale.','标记缩放比例。','標記縮放比例。'),
 'mode.legacy2D.yOffset':('Marker vertical offset in blocks.','标记竖直偏移，单位为格。','標記垂直偏移，單位為格。'),
 'mode.legacy2D.backgroundAlpha':('Black marker background opacity.','标记黑色背景的不透明度。','標記黑色背景的不透明度。'),
 'colorMode':('Choose fixed or rainbow color.','选择固定或彩虹颜色。','選擇固定或彩虹顏色。'),
 'colorMode.static':('Use the configured fixed color.','使用设定的固定颜色。','使用設定的固定顏色。'),
 'colorMode.static.color':('Fixed model and marker color.','模型与标记的固定颜色。','模型與標記的固定顏色。'),
 'colorMode.rainbow':('Cycle rainbow colors with the original alpha of 50.','循环显示彩虹颜色，保留原版 50 的透明度值。','循環顯示彩虹顏色，保留原版 50 的透明度值。'),
}
LABELS={'ItemESP':('物品透视','物品透視'),'ShowArrows':('显示箭','顯示箭'),'RegularArrows':('普通箭','一般箭'),'SpectralArrows':('光灵箭','追跡之箭'),'ArrowsWithEffects':('药水箭','藥水箭'),'ShowTridents':('显示三叉戟','顯示三叉戟'),'MergeIntersecting':('合并相交方框','合併交疊方框')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';d=json.loads(path.read_text(encoding='utf8'));suffix=lambda k:k if k=='description'else k+'.description'
  # Glow controls use the already translated shared StorageESP style semantics.
  prefix='liquidbounce.module.storageESP.mode.glow.style.'
  shared={k.replace('liquidbounce.module.storageESP','liquidbounce.module.itemEsp'):v for k,v in d['descriptions'].items()if k.startswith(prefix)}
  shared_en={k.replace('liquidbounce.module.storageESP','liquidbounce.module.itemEsp'):v for k,v in d.get('englishDescriptions',{}).items()if k.startswith(prefix)}
  put_module(d,'liquidbounce.module.itemEsp','ItemESP',{'':LABELS['ItemESP'][index-1],**{suffix(k):v[index]for k,v in TEXTS.items()}},{suffix(k):v[0]for k,v in TEXTS.items()})
  d['descriptions'].update(shared)
  if 'englishDescriptions'in d:d['englishDescriptions'].update(shared_en)
  d['labels'].update({k:v[index-1]for k,v in LABELS.items()});path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().directory)
