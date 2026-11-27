"""Complete explicit PotionFX simplified/traditional display metadata."""
from pathlib import Path
import argparse,json
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1];PREFIX='liquidbounce.module.potionFX'
LABELS={'PotionFX':'药水特效','Players':'玩家','SplashPotion':'喷溅药水','LingeringPotion':'滞留药水','MainEffect':'主要效果','SecondEffect':'次要效果','SecondEffects':'次要效果','Flash':'闪光','Effect':'效果','Enabled':'启用','Radius':'半径','ExtraRadius':'额外半径','RotationSpeed':'旋转速度','Lifetime':'寿命','FadeStart':'渐隐起点','AnimTime':'动画时间','AnimAcceleration':'动画加速','CanBeCovered':'可被遮挡','Source':'来源','Builtin':'内置','Custom':'自定义','Preset':'预设','File':'文件','Dashed':'虚线','Solid':'实线','Runes':'符文','Atlas':'图集','Cracked':'裂纹','Neuron':'神经元','Hexagon':'六边形','Stardust':'星尘'}
LABELS_TW={'PotionFX':'藥水特效','Players':'玩家','SplashPotion':'飛濺藥水','LingeringPotion':'滯留藥水','MainEffect':'主要效果','SecondEffect':'次要效果','SecondEffects':'次要效果','Flash':'閃光','Effect':'效果','Enabled':'啟用','Radius':'半徑','ExtraRadius':'額外半徑','RotationSpeed':'旋轉速度','Lifetime':'壽命','FadeStart':'漸隱起點','AnimTime':'動畫時間','AnimAcceleration':'動畫加速','CanBeCovered':'可被遮擋','Source':'來源','Builtin':'內建','Custom':'自訂','Preset':'預設','File':'檔案','Dashed':'虛線','Solid':'實線','Runes':'符文','Atlas':'圖集','Cracked':'裂紋','Neuron':'神經元','Hexagon':'六邊形','Stardust':'星塵'}
HELP={'':'按原始药水颜色绘制玩家脚下图案、喷溅扩张图案和滞留云图案；三个组可独立启用。','Players':'默认开启；仅为存在可见状态效果颜色的玩家绘制，位置按帧插值。','SplashPotion':'默认开启；只处理原始 2002 喷溅事件，并要求附近实际存在喷溅药水实体。','LingeringPotion':'默认关闭；绘制实际滞留云半径及颜色，位置保持实体当前位置。','Enabled':'独立启用本组；关闭喷溅组会清除已保存的喷溅特效。','MainEffect':'主要水平贴图及其动画、来源设置。','SecondEffect':'可选的第二层玩家贴图，默认关闭。','SecondEffects':'独立配置可选第二层图案和相机朝向闪光。','Flash':'面向相机的原始 glow 贴图；颜色不随喷溅尾部渐隐。','Effect':'可选的第二层水平图案，默认关闭；喷溅及滞留时位于主层下方 0.005 方块。','Radius':'图案或闪光的半径，绘制边长为半径两倍。','ExtraRadius':'在主图案或实际滞留云半径上增加的半径。','RotationSpeed':'每游戏刻的旋转角度，可用负值反向旋转。','Lifetime':'从下限游戏刻完成指数缓出扩张，在上限游戏刻过期；更改值保留已有到期时间。','FadeStart':'喷溅尾部开始渐隐的寿命比例，默认 0.9；1 表示到期前不渐隐。','AnimTime':'闪光原始 QuadInOut 缓动的时间尺度，默认 4 游戏刻；滞留云的输入年龄上限为此值两倍。','AnimAcceleration':'第二层喷溅图案的扩张进度乘数，默认 1.25；乘积限制在 0～1。','CanBeCovered':'默认开启；使用实际场景深度允许方块遮挡效果。','Source':'分别选择主层或第二层的内置或自定义贴图；第二层来源尚未可用时整组暂停绘制。','Builtin':'使用原始 LiquidBounce PNG 贴图。','Custom':'异步读取所选 PNG/JPG；无效或解码失败保留本槽上次成功的贴图。第二层关闭时不会开始读取新的自定义文件。','Preset':'选择原始四张主层或四张第二层贴图之一。','File':'选择 PNG、JPG 或 JPEG 文件；每个组的主层和第二层独立保存。'}
HELP_TW={'':'依原始藥水色彩繪製玩家腳下圖案、飛濺擴張圖案和滯留雲圖案；三組可獨立啟用。','Players':'預設開啟；僅為存在可見狀態效果色彩的玩家繪製，位置依影格插值。','SplashPotion':'預設開啟；只處理原始 2002 飛濺事件，並要求附近實際存在飛濺藥水實體。','LingeringPotion':'預設關閉；繪製實際滯留雲半徑及色彩，位置維持實體目前位置。','Enabled':'獨立啟用本組；關閉飛濺組會清除已儲存的飛濺特效。','MainEffect':'主要水平貼圖及其動畫、來源設定。','SecondEffect':'可選的第二層玩家貼圖，預設關閉。','SecondEffects':'獨立設定可選第二層圖案和朝向相機的閃光。','Flash':'朝向相機的原始 glow 貼圖；色彩不隨飛濺尾部漸隱。','Effect':'可選的第二層水平圖案，預設關閉；飛濺及滯留時位於主層下方 0.005 方塊。','Radius':'圖案或閃光的半徑，繪製邊長為半徑兩倍。','ExtraRadius':'在主圖案或實際滯留雲半徑上增加的半徑。','RotationSpeed':'每遊戲刻的旋轉角度，可用負值反向旋轉。','Lifetime':'從下限遊戲刻完成指數緩出擴張，在上限遊戲刻到期；變更值保留既有到期時間。','FadeStart':'飛濺尾部開始漸隱的壽命比例，預設 0.9；1 表示到期前不漸隱。','AnimTime':'閃光原始 QuadInOut 緩動的時間尺度，預設 4 遊戲刻；滯留雲的輸入年齡上限為此值兩倍。','AnimAcceleration':'第二層飛濺圖案的擴張進度乘數，預設 1.25；乘積限制在 0～1。','CanBeCovered':'預設開啟；使用實際場景深度允許方塊遮擋效果。','Source':'分別選取主層或第二層的內建或自訂貼圖；第二層來源尚未可用時整組暫停繪製。','Builtin':'使用原始 LiquidBounce PNG 貼圖。','Custom':'非同步讀取所選 PNG/JPG；無效或解碼失敗保留此槽上次成功的貼圖。第二層關閉時不會開始讀取新的自訂檔案。','Preset':'選取原始四張主層或四張第二層貼圖之一。','File':'選取 PNG、JPG 或 JPEG 檔案；每組的主層和第二層獨立儲存。'}
def paths():
 out=['']
 for name in ['Players','SplashPotion','LingeringPotion']:
  out.extend([name,name+'.Enabled',name+'.MainEffect',name+'.CanBeCovered'])
  main=name+'.MainEffect';out.extend(main+'.'+n for n in (['Radius','RotationSpeed']if name=='Players'else['Lifetime','Radius','RotationSpeed','FadeStart']if name=='SplashPotion'else['ExtraRadius','RotationSpeed']))
  if name=='Players':second=name+'.SecondEffect';out.extend([second,second+'.Enabled',second+'.RotationSpeed',second+'.ExtraRadius'])
  else:
   group=name+'.SecondEffects';second=group+'.Effect';out.extend([group,group+'.Flash',group+'.Flash.Enabled',group+'.Flash.AnimTime',group+'.Flash.Radius',second,second+'.Enabled',second+'.RotationSpeed',second+'.ExtraRadius'])
   if name=='SplashPotion':out.append(second+'.AnimAcceleration')
  for group in [main,second]:out.extend(group+'.Source'+n for n in ['', '.Builtin','.Builtin.Preset','.Custom','.Custom.File'])
 return out
def descriptions(language):
 help=HELP_TW if language=='zh_tw'else HELP
 return {PREFIX+(''.join('.'+n[:1].lower()+n[1:]for n in p.split('.'))if p else'')+'.description':(f'PotionFX {p or "module"}: original independent group setting; inactive sources and values are retained.'if language=='en_us'else help[p.rsplit('.',1)[-1]] )for p in paths()}
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(LABELS_TW if language=='zh_tw'else LABELS);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',required=True);args=a.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,ROOT/args.directory/f'display_{lang}.json')
