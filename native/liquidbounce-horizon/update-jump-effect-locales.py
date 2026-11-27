"""Full JumpEffect display metadata, explicit dictionary only; never changes canonical locales."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'update-breadcrumbs-locales.py').is_file()else HERE.parents[1]
ROOT=NATIVE.parents[1]
B=runpy.run_path(str(NATIVE/'update-breadcrumbs-locales.py'))
PREFIX='liquidbounce.module.jumpEffect'
LABELS={'JumpEffect':'跳跃特效','Simple':'渐变圆环','Image':'图片','EndRadius':'结束半径','AnimCurve':'动画缓动','HueOffsetAnim':'动态色相偏移','Lifetime':'动画与寿命','CanBeCovered':'可被遮挡','Colors':'颜色','InnerColor':'内侧颜色','OuterColor':'外侧颜色','RotationSpeed':'旋转速度','Source':'图片来源','Builtin':'内置','Custom':'自定义','Preset':'预设','File':'图片文件','LiquidBounceWithLogo':'LiquidBounce（带标志）'}
LABELS_TW={'JumpEffect':'跳躍特效','Simple':'漸層圓環','Image':'圖片','EndRadius':'結束半徑','AnimCurve':'動畫緩動','HueOffsetAnim':'動態色相偏移','Lifetime':'動畫與壽命','CanBeCovered':'可被遮擋','Colors':'色彩','InnerColor':'內側色彩','OuterColor':'外側色彩','RotationSpeed':'旋轉速度','Source':'圖片來源','Builtin':'內建','Custom':'自訂','Preset':'預設','File':'圖片檔案','LiquidBounceWithLogo':'LiquidBounce（附標誌）'}
HELP={
 '':'在玩家实际跳跃的位置绘制短暂效果；禁用或更换模式时清除现有圆环。',
 'Mode':'选择渐变圆环或原始图片效果；默认渐变圆环。',
 'Simple':'在水平面绘制随年龄扩张和渐隐的渐变圆环。',
 'Image':'在水平面绘制旋转的原始图片，可使用自定义 PNG/JPG。',
 'EndRadius':'最终内侧与外侧半径，单位方块；默认 0.15～0.8。',
 'AnimCurve':'圆环扩张使用的缓动函数，默认二次缓出。',
 'HueOffsetAnim':'按寿命进度改变色相的角度；默认 -63°。',
 'Lifetime':'前值是扩张动画时长，后值是效果寿命，单位游戏刻；动画结束后渐隐。',
 'CanBeCovered':'启用真实场景深度测试，让方块遮住效果。',
 'Colors':'效果内外两端的 ARGB 颜色。',
 'InnerColor':'效果内侧 ARGB 颜色，默认透明度 50。',
 'OuterColor':'效果外侧 ARGB 颜色，默认透明度 150。',
 'RotationSpeed':'图片每游戏刻旋转的角度；负值反向。',
 'Source':'选择两张原始内置图片之一，或自定义 PNG/JPG。',
 'Builtin':'使用 LiquidBounce 原始跳跃效果贴图。',
 'Custom':'从本地图片文件载入贴图；读取失败保留上次成功图片。',
 'Preset':'选择普通 LiquidBounce 或带标志的原始图片。',
 'File':'选择 PNG、JPG 或 JPEG 图片；非文件路径不替换已有贴图。',
 'Render':'普通模式直接绘制圆环；着色器模式应用选中的完整 ShaderLayer 样式。',
 'Plain':'直接以渐变颜色绘制效果。',
 'Shader':'捕获圆环剪影并应用距离场着色器。',
 'Style':'选择原始九种 ShaderLayer 样式。',
}
HELP_TW={
 '':'在玩家實際跳躍的位置繪製短暫效果；停用或更換模式時清除現有圓環。','Mode':'選取漸層圓環或原始圖片效果；預設漸層圓環。','Simple':'在水平面繪製隨年齡擴張與漸隱的漸層圓環。','Image':'在水平面繪製旋轉的原始圖片，可使用自訂 PNG/JPG。','EndRadius':'最終內側與外側半徑，單位為方塊；預設 0.15～0.8。','AnimCurve':'圓環擴張使用的緩動函數，預設二次緩出。','HueOffsetAnim':'依壽命進度改變色相的角度；預設 -63°。','Lifetime':'前值為擴張動畫時間，後值為效果壽命，單位遊戲刻；動畫結束後漸隱。','CanBeCovered':'啟用真實場景深度測試，讓方塊遮住效果。','Colors':'效果內外兩端的 ARGB 色彩。','InnerColor':'效果內側 ARGB 色彩，預設透明度 50。','OuterColor':'效果外側 ARGB 色彩，預設透明度 150。','RotationSpeed':'圖片每遊戲刻旋轉的角度；負值反向。','Source':'選取兩張原始內建圖片之一，或自訂 PNG/JPG。','Builtin':'使用 LiquidBounce 原始跳躍效果貼圖。','Custom':'從本機圖片檔案載入貼圖；讀取失敗保留上次成功圖片。','Preset':'選取一般 LiquidBounce 或附標誌的原始圖片。','File':'選取 PNG、JPG 或 JPEG 圖片；非檔案路徑不取代已有貼圖。','Render':'一般模式直接繪製圓環；著色器模式套用選取的完整 ShaderLayer 樣式。','Plain':'直接以漸層色彩繪製效果。','Shader':'擷取圓環剪影並套用距離場著色器。','Style':'選取原始九種 ShaderLayer 樣式。'}
def paths():
 out=['','Mode']
 for mode in ['Simple','Image']:
  p='Mode.'+mode;out.append(p)
  out.extend(p+'.'+n for n in ['EndRadius','AnimCurve','HueOffsetAnim','Lifetime','CanBeCovered'])
  if mode=='Simple':
   out.extend(p+'.'+n for n in ['Colors','Colors.InnerColor','Colors.OuterColor','Render','Render.Plain','Render.Shader','Render.Shader.Style'])
   for style,fields in zip(B['NAMES'],B['FIELDS']):out.append(p+'.Render.Shader.Style.'+style);out.extend(p+'.Render.Shader.Style.'+style+'.'+n for n in ['Opacity','Fill']+fields)
  else:out.extend(p+'.'+n for n in ['RotationSpeed','Source','Source.Builtin','Source.Builtin.Preset','Source.Custom','Source.Custom.File','Colors','Colors.InnerColor','Colors.OuterColor'])
 return out
def descriptions(language):
 help=HELP_TW if language=='zh_tw'else HELP;params=B['PARAMS_TW']if language=='zh_tw'else B['PARAMS'];result={}
 for p in paths():
  n=p.rsplit('.',1)[-1];k=PREFIX+(''.join('.'+n[:1].lower()+n[1:]for n in p.split('.'))if p else'')+'.description'
  v=help.get(n,params.get(n,('使用原始 '+n+' ShaderLayer 样式。')if language!='zh_tw'else('使用原始 '+n+' ShaderLayer 樣式。')))
  if language=='en_us':v=f'JumpEffect {p or "module"}: original source setting; inactive mode values are retained.'
  result[k]=v.replace('轨迹','圆环').replace('軌跡','圓環')
 return result
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(LABELS_TW if language=='zh_tw'else LABELS);data['labels'].update(B['LABELS_TW']if language=='zh_tw'else B['LABELS']);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',required=True);args=a.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,ROOT/args.directory/f'display_{language}.json')
