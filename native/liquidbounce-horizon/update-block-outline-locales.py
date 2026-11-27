from pathlib import Path
import argparse,json
ROOT=Path(__file__).resolve().parents[2]
LABELS={'zh_cn':{'BlockOutline':'方块高亮','BlockOverlay':'方块覆盖层','SideOnly':'仅命中面','Outline':'轮廓颜色','Slide':'滑动动画','Time':'时长'},'zh_tw':{'BlockOutline':'方塊醒目提示','BlockOverlay':'方塊覆蓋層','SideOnly':'僅命中面','Outline':'外框色彩','Slide':'滑動動畫','Time':'時間'}}
HELP={
'en_us':{'sideOnly':'Highlight the connected region of the hit face. A single box uses only its selected side.','color':'Face fill ARGB color, including alpha.','outline':'Outline ARGB color, including alpha.','slide.enabled':'Animate transitions between single-box targets. Complex shapes reset the transition and render their actual exposed surface.','slide.time':'Slide duration in milliseconds. Default 150; UI range 1–1000.','slide.easing':'The source easing curve used for the slide transition.'},
'zh_cn':{'sideOnly':'高亮命中面所属的连通区域；单盒形状仅显示选中的一面。','color':'面的填充 ARGB 颜色，包含透明度。','outline':'轮廓的 ARGB 颜色，包含透明度。','slide.enabled':'在单盒目标之间平滑移动；复杂形状会重置动画并绘制实际外表面。','slide.time':'滑动时长，单位毫秒。默认 150，界面范围 1～1000。','slide.easing':'滑动动画使用的原始缓动曲线。'},
'zh_tw':{'sideOnly':'醒目提示命中面所屬的連通區域；單盒形狀僅顯示選中的一面。','color':'面的填滿 ARGB 色彩，包含透明度。','outline':'外框的 ARGB 色彩，包含透明度。','slide.enabled':'在單盒目標之間平滑移動；複雜形狀會重設動畫並繪製實際外表面。','slide.time':'滑動時間，單位毫秒。預設 150，介面範圍 1～1000。','slide.easing':'滑動動畫使用的原始緩動曲線。'}}
def descriptions(language):
 source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 result={f'liquidbounce.module.blockOutline.{k}.description':v for k,v in HELP[language].items()};key='liquidbounce.module.blockOutline.description';result[key]=source[key];return result
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(LABELS[language]);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,ROOT/a.directory/f'display_{language}.json')
