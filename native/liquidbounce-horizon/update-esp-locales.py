"""Complete explicit ESP Simplified/Traditional settings and original style help."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'update-tracers-locales.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]
TRACERS=runpy.run_path(str(NATIVE/'update-tracers-locales.py'));STORAGE=runpy.run_path(str(NATIVE/'update-storage-esp-locales.py'))
LABELS={'ESP':'实体透视','2D':'二维方框','Legacy2D':'经典二维标记','Expand':'扩展','MergeIntersecting':'合并相交方框','Outline':'轮廓','Corner':'角框','Gap':'缺口','Border':'黑色边框','HealthBar':'生命条','Spacing':'间距','BackgroundAlpha':'背景透明度','Invisible':'隐身颜色','Friends':'好友颜色','Objects':'其他实体','ObjectColor':'其他实体颜色','EndCrystals':'末地水晶','Projectiles':'投射物','Vehicles':'载具','Thickness':'粗细','YOffset':'纵向偏移'}
LABELS_TW={'ESP':'實體透視','2D':'二維方框','Legacy2D':'經典二維標記','Expand':'擴展','MergeIntersecting':'合併相交方框','Outline':'輪廓','Corner':'角框','Gap':'缺口','Border':'黑色邊框','HealthBar':'生命條','Spacing':'間距','BackgroundAlpha':'背景不透明度','Invisible':'隱形色彩','Friends':'好友色彩','Objects':'其他實體','ObjectColor':'其他實體色彩','EndCrystals':'終界水晶','Projectiles':'投射物','Vehicles':'載具','Thickness':'粗細','YOffset':'縱向偏移'}
# English, Simplified Chinese, Traditional Chinese, keyed exactly like the runtime tree.
TEXTS={
'':('Highlight visual targets and selected objects with four original ESP modes.','使用四种原始模式突出显示全局视觉目标和选定的其他实体。','以四種原始模式突顯全域視覺目標與選取的其他實體。'),
'Mode':('Choose Box, 2D, Legacy2D or real model Glow; default Glow.','选择方框、二维方框、经典二维标记或真实模型辉光；默认辉光。','選擇方框、二維方框、經典二維標記或真實模型輝光；預設輝光。'),
'Mode.Box.Expand':('Inflate the tick-cached pose box; default 0.05, range 0–0.5 blocks.','每刻缓存姿态包围盒的外扩距离；默认 0.05，范围 0～0.5 方块。','每刻快取姿態邊界盒的外擴距離；預設 0.05，範圍 0～0.5 方塊。'),
'Mode.Box.Outline':('Draw box edges with alpha 100/255; fill always uses 50/255.','绘制透明度固定为 100/255 的边线；方框填充固定为 50/255。','繪製不透明度固定為 100/255 的邊線；方框填滿固定為 50/255。'),
'Mode.Box.MergeIntersecting':('Merge strictly intersecting boxes with the same full ARGB color.','按完整 ARGB 颜色合并严格相交的方框；仅接触边界的方框不会合并。','依完整 ARGB 色彩合併嚴格相交的方框；僅接觸邊界的方框不會合併。'),
'Mode.2D.Expand':('Inflate the tick-cached box before projecting all eight corners.','投影八个顶点前扩展每刻缓存的包围盒；默认 0.05 方块。','投影八個頂點前擴展每刻快取的邊界盒；預設 0.05 方塊。'),
'Mode.2D.Fill':('Fill the projected rectangle at alpha 50/255.','以固定 50/255 的透明度填充屏幕方框。','以固定 50/255 的不透明度填滿螢幕方框。'),
'Mode.2D.Outline':('Draw the colored rectangle edges.','绘制彩色矩形边线。','繪製彩色矩形邊線。'),
'Mode.2D.Outline.Enabled':('Enable colored rectangle edges.','启用彩色边线。','啟用彩色邊線。'),
'Mode.2D.Outline.Thickness':('Colored edge thickness, divided by the raw GUI scale setting; range 1–9.','彩色边线粗细除以原始界面缩放设置；默认 1，范围 1～9。','彩色邊線粗細除以原始介面縮放設定；預設 1，範圍 1～9。'),
'Mode.2D.Corner':('Replace full edges with separated corner segments.','将完整边线改成带缺口的角框。','將完整邊線改成帶缺口的角框。'),
'Mode.2D.Corner.Enabled':('Enable separated corner segments.','启用角框。','啟用角框。'),
'Mode.2D.Corner.Gap':('Gap between corner segments as a percentage; default 50, range 1–100.','角框中间缺口的百分比；默认 50，范围 1～100。','角框中央缺口的百分比；預設 50，範圍 1～100。'),
'Mode.2D.Border':('Draw black borders around edges and the health bar.','在彩色边线和生命条外绘制黑色边框。','在彩色邊線與生命條外繪製黑色邊框。'),
'Mode.2D.Border.Enabled':('Enable black borders.','启用黑色边框。','啟用黑色邊框。'),
'Mode.2D.Border.Thickness':('Black border thickness divided by the raw GUI scale setting; range 1–9.','黑色边框粗细除以原始界面缩放设置；默认 1，范围 1～9。','黑色邊框粗細除以原始介面縮放設定；預設 1，範圍 1～9。'),
'Mode.2D.HealthBar':('Draw actual living health beside the box.','在方框左侧显示生物的实际生命值，不含吸收生命值。','在方框左側顯示生物的實際生命值，不含吸收生命值。'),
'Mode.2D.HealthBar.Enabled':('Enable the living entity health bar.','启用生物生命条。','啟用生物生命條。'),
'Mode.2D.HealthBar.Spacing':('Health bar spacing divided by the raw GUI scale; default 2, range 0–32.','生命条间距除以原始界面缩放设置；默认 2，范围 0～32。','生命條間距除以原始介面縮放設定；預設 2，範圍 0～32。'),
'Mode.Legacy2D.Scale':('Yaw-facing marker scale; default 0.1, range 0.02–0.3.','仅随相机偏航转动的经典标记缩放；默认 0.1，范围 0.02～0.3。','僅隨相機偏航轉動的經典標記縮放；預設 0.1，範圍 0.02～0.3。'),
'Mode.Legacy2D.YOffset':('Vertical marker offset; default 0, range −1–1.','经典标记的纵向偏移；默认 0，范围 −1～1。','經典標記的縱向偏移；預設 0，範圍 −1～1。'),
'Mode.Legacy2D.BackgroundAlpha':('Black marker background alpha; default 150, range 0–255.','黑色标记背景的透明度；默认 150，范围 0～255。','黑色標記背景的不透明度；預設 150，範圍 0～255。'),
'Mode.Glow.Style':('Original Vanilla Sobel and two blur passes, or nine styles; default Smooth.','使用原版边缘检测与两次模糊，或九种原始后处理样式；默认柔光。','使用原版邊緣偵測與兩次模糊，或九種原始後製樣式；預設柔光。'),
'ColorMode':('After hurt, invisible, friend and player tag colors, use the selected color mode.','受伤红色、隐身颜色、好友颜色和玩家标签颜色依次优先，之后使用选定的颜色模式。','受傷紅色、隱形色彩、好友色彩與玩家標籤色彩依序優先，之後使用選取的色彩模式。'),
'ColorMode.Static.Color':('Static ARGB color; default white with alpha 100.','静态 ARGB 颜色；默认透明度 100 的白色。','固定 ARGB 色彩；預設不透明度 100 的白色。'),
'Invisible':('Living invisible color after hurt red; default orange.','生物隐身时使用的颜色，优先级低于受伤红色；默认橙色。','生物隱形時使用的色彩，優先順序低於受傷紅色；預設橙色。'),
'Friends':('Friend player color; alpha zero lets the tag or color mode apply.','好友玩家颜色；透明度为零时继续使用标签颜色或颜色模式，默认绿色。','好友玩家色彩；不透明度為零時繼續使用標籤色彩或色彩模式，預設綠色。'),
'MaximumDistance':('Strict camera distance to current position; default 128, range 1–512 blocks.','按相机到实体当前位置的距离严格筛选，小于该距离才显示；默认 128，范围 1～512 方块。','依相機到實體目前位置的距離嚴格篩選，小於該距離才顯示；預設 128，範圍 1～512 方塊。'),
'Objects':('Select end crystals, projectiles and vehicles; exclude the own ridden vehicle.','额外选择末地水晶、投射物或载具，默认不选；始终排除自己乘坐的载具。','額外選取終界水晶、投射物或載具，預設不選；一律排除自己乘坐的載具。'),
'ObjectColor':('Color for selected nonliving objects; default white with alpha 100.','选中非生物实体的颜色；默认透明度 100 的白色。','選取非生物實體的色彩；預設不透明度 100 的白色。')}
def key(p):return 'liquidbounce.module.esp'+(''.join('.'+n[0].lower()+n[1:]for n in p.split('.'))if p else'')+'.description'
def descriptions(lang):
 result={}
 for k,v in TRACERS['descriptions'](lang).items():
  if '.colorMode.'in k:result[k.replace('.tracers.','.esp.')]=v
 for k,v in STORAGE['descriptions'](lang).items():
  if '.mode.glow.style.'in k:result[k.replace('.storageESP.','.esp.')]=v.replace('存储','实体').replace('儲存','實體').replace('StorageESP','ESP')
 i={'en_us':0,'zh_cn':1,'zh_tw':2}[lang]
 result.update({key(p):v[i]for p,v in TEXTS.items()});return result
def update(lang,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(STORAGE['BASE']['LABELS_TW']if lang=='zh_tw'else STORAGE['BASE']['LABELS']);data['labels'].update(LABELS_TW if lang=='zh_tw'else LABELS);data['descriptions'].update(descriptions(lang))
 if lang=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,ROOT/a.directory/f'display_{lang}.json')
