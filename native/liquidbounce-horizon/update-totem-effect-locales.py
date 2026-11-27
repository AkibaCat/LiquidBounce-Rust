"""Explicit TotemEffect metadata; writes only the directory supplied by the parent."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'update-breadcrumbs-locales.py').is_file()else HERE.parents[1]
ROOT=NATIVE.parents[1];B=runpy.run_path(str(NATIVE/'update-breadcrumbs-locales.py'));PREFIX='liquidbounce.module.totemEffect'
LABELS={'TotemEffect':'图腾特效','Shockwave':'冲击波','Soul':'灵魂','Lifetime':'寿命','Fade':'渐隐起点','AnimCurve':'动画缓动','CanBeCovered':'可被遮挡','radius':'半径','Colors':'颜色','SyncColors':'同步颜色','InnerColor':'内侧颜色','OuterColor':'外侧颜色','Halo':'光环','YMotion':'垂直移动','SoulYMotion':'垂直移动系数','AnimBy':'动画依据','Progress':'进度','Age':'年龄'}
LABELS_TW={'TotemEffect':'圖騰特效','Shockwave':'衝擊波','Soul':'靈魂','Lifetime':'壽命','Fade':'漸隱起點','AnimCurve':'動畫緩動','CanBeCovered':'可被遮擋','radius':'半徑','Colors':'色彩','SyncColors':'同步色彩','InnerColor':'內側色彩','OuterColor':'外側色彩','Halo':'光環','YMotion':'垂直移動','SoulYMotion':'垂直移動係數','AnimBy':'動畫依據','Progress':'進度','Age':'年齡'}
HELP={'':'在符合视觉目标设置的实体触发图腾保护时，保存其位置与旋转并绘制短暂效果。','Mode':'选择冲击波或站立灵魂；切换模式会清除已有快照。','Shockwave':'在实体高度中点绘制扩张的水平渐变圆环。','Soul':'按触发时的俯仰和偏航绘制站立线框人物，可向上移动并显示光环。','Lifetime':'快照寿命，默认 20 游戏刻；更改此值清除已有快照，支持旧名称 lifetime。','Fade':'按缓动后的动画进度开始渐隐，默认 0.7；1 表示寿命内不渐隐。','AnimCurve':'扩张和进度动画的缓动函数，默认指数缓出。','CanBeCovered':'使用真实场景深度，让方块遮住特效。','radius':'冲击波最终内外半径，默认 2～2.2 方块；原设置名保留小写 radius。','Colors':'效果内外颜色；透明度由寿命渐隐计算覆盖。','SyncColors':'启用后外侧颜色与内侧一致。','InnerColor':'冲击波内侧和灵魂填充颜色，默认亮绿色。','OuterColor':'冲击波外侧和灵魂轮廓颜色，默认黄绿色。','Halo':'在灵魂上方绘制原始 144×12 分段的环形光环。','YMotion':'启用灵魂垂直移动；默认开启。','Enabled':'启用或关闭本组功能。','SoulYMotion':'垂直动画乘数，默认 0.75；按年龄时为每游戏刻的移动量。','AnimBy':'使用原始游戏刻年龄或缓动后的进度计算垂直移动，默认年龄。','Render':'普通绘制或完整 ShaderLayer 着色器效果。','Plain':'直接绘制渐变圆环。','Shader':'捕获圆环剪影并应用原始距离场样式。','Style':'选择原始九种 ShaderLayer 样式。'}
HELP_TW={'':'在符合視覺目標設定的實體觸發圖騰保護時，儲存其位置與旋轉並繪製短暫效果。','Mode':'選取衝擊波或站立靈魂；切換模式會清除已有快照。','Shockwave':'在實體高度中點繪製擴張的水平漸層圓環。','Soul':'依觸發時的俯仰與偏航繪製站立線框人物，可向上移動並顯示光環。','Lifetime':'快照壽命，預設 20 遊戲刻；變更此值清除已有快照，支援舊名稱 lifetime。','Fade':'依緩動後的動畫進度開始漸隱，預設 0.7；1 表示壽命內不漸隱。','AnimCurve':'擴張及進度動畫的緩動函數，預設指數緩出。','CanBeCovered':'使用真實場景深度，讓方塊遮住特效。','radius':'衝擊波最終內外半徑，預設 2～2.2 方塊；原設定名稱保留小寫 radius。','Colors':'效果內外色彩；透明度由壽命漸隱計算覆寫。','SyncColors':'啟用後外側色彩與內側一致。','InnerColor':'衝擊波內側和靈魂填充色彩，預設亮綠色。','OuterColor':'衝擊波外側和靈魂輪廓色彩，預設黃綠色。','Halo':'在靈魂上方繪製原始 144×12 分段的環形光環。','YMotion':'啟用靈魂垂直移動；預設開啟。','Enabled':'啟用或關閉本組功能。','SoulYMotion':'垂直動畫乘數，預設 0.75；依年齡時為每遊戲刻的移動量。','AnimBy':'使用原始遊戲刻年齡或緩動後的進度計算垂直移動，預設年齡。','Render':'一般繪製或完整 ShaderLayer 著色器效果。','Plain':'直接繪製漸層圓環。','Shader':'擷取圓環剪影並套用原始距離場樣式。','Style':'選取原始九種 ShaderLayer 樣式。'}
def paths():
 out=['','Mode']
 for mode in ['Shockwave','Soul']:
  p='Mode.'+mode;out.append(p);out.extend(p+'.'+n for n in ['Lifetime','Fade','AnimCurve','CanBeCovered'])
  if mode=='Shockwave':out.append(p+'.radius')
  else:out.extend(p+'.'+n for n in ['Halo','YMotion','YMotion.Enabled','YMotion.SoulYMotion','YMotion.AnimBy'])
  out.extend(p+'.'+n for n in ['Colors','Colors.SyncColors','Colors.InnerColor','Colors.OuterColor'])
  if mode=='Shockwave':
   out.extend(p+'.'+n for n in ['Render','Render.Plain','Render.Shader','Render.Shader.Style'])
   for style,fields in zip(B['NAMES'],B['FIELDS']):out.append(p+'.Render.Shader.Style.'+style);out.extend(p+'.Render.Shader.Style.'+style+'.'+n for n in ['Opacity','Fill']+fields)
 return out
def descriptions(language):
 help=HELP_TW if language=='zh_tw'else HELP;params=B['PARAMS_TW']if language=='zh_tw'else B['PARAMS'];out={}
 for p in paths():
  n=p.rsplit('.',1)[-1];k=PREFIX+(''.join('.'+n[:1].lower()+n[1:]for n in p.split('.'))if p else'')+'.description'
  out[k]=f'TotemEffect {p or "module"}: original source setting; inactive mode values are retained.'if language=='en_us'else help.get(n,params.get(n,('使用原始 '+n+' ShaderLayer 樣式。')if language=='zh_tw'else('使用原始 '+n+' ShaderLayer 样式。')))
 return out
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));data['labels'].update(B['LABELS_TW']if language=='zh_tw'else B['LABELS']);data['labels'].update(LABELS_TW if language=='zh_tw'else LABELS);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',required=True);args=a.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,ROOT/args.directory/f'display_{language}.json')
