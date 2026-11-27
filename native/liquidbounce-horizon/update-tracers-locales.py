"""Complete explicit Simplified/Traditional Tracers and shared shader descriptions."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
BASE=runpy.run_path(str(HERE/'update-breadcrumbs-locales.py'))
LABELS={'Tracers':'实体射线','ColorMode':'颜色模式','Distance':'距离','Health':'生命值','Static':'静态','Hue':'色相','Brightness':'亮度','Alpha':'透明度','LineWidth':'线宽','MaximumDistance':'最大距离'}
LABELS_TW={'Tracers':'實體射線','ColorMode':'色彩模式','Distance':'距離','Health':'生命值','Static':'固定','Hue':'色相','Brightness':'亮度','Alpha':'透明度','LineWidth':'線寬','MaximumDistance':'最遠距離'}
CN={'':'绘制从视线前方到可视目标的射线；目标由全局视觉目标设置和实体合并决定。','ColorMode':'优先使用好友蓝色和实体标签颜色，再应用选中的颜色模式。','ColorMode.Distance.Saturation':'按相机到实体的当前距离取色时使用的饱和度。','ColorMode.Distance.Brightness':'距离颜色的亮度。','ColorMode.Distance.Hue':'距离到色相的曲线，默认控制点为 (0,0)、(100,120)、(200,120)，张力 0.4。','ColorMode.Distance.Alpha':'距离颜色的不透明度，默认 1。','ColorMode.Health.Alpha':'生命值颜色的不透明度，默认 255；红绿比例使用实际生命值，不含吸收。','ColorMode.Static.Color':'静态 ARGB 颜色，默认不透明的蓝青色。','LineWidth':'射线宽度，单位屏幕像素；默认 1，界面范围 1～16。','MaximumDistance':'相机到实体当前位置的最大距离，默认 128 方块；界面范围 1～512。','Render':'普通射线，或应用九种原始距离场着色器样式。','Render.Shader.Style':'选择一种原始后处理样式；射线内部填充默认 1。'}
TW={'':'繪製從視線前方到可見目標的射線；目標由全域視覺目標設定與實體合併決定。','ColorMode':'優先使用好友藍色與實體標籤色彩，再套用選取的色彩模式。','ColorMode.Distance.Saturation':'依相機到實體的目前距離取色時使用的飽和度。','ColorMode.Distance.Brightness':'距離色彩的亮度。','ColorMode.Distance.Hue':'距離對應色相的曲線，預設控制點為 (0,0)、(100,120)、(200,120)，張力為 0.4。','ColorMode.Distance.Alpha':'距離色彩的不透明度，預設 1。','ColorMode.Health.Alpha':'生命值色彩的不透明度，預設 255；紅綠比例使用實際生命值，不含吸收。','ColorMode.Static.Color':'固定 ARGB 色彩，預設為不透明的藍青色。','LineWidth':'射線寬度，單位為螢幕像素；預設 1，介面範圍 1～16。','MaximumDistance':'相機到實體目前位置的最遠距離，預設 128 方塊；介面範圍 1～512。','Render':'一般射線，或套用九種原始距離場著色器樣式。','Render.Shader.Style':'選取一種原始後處理樣式；射線內部填滿預設為 1。'}
EN={'':'Draw lines from the camera eye vector to visible living entities.','ColorMode':'Friend blue takes priority, followed by entity tags, then the selected color mode.','ColorMode.Distance.Saturation':'Distance color saturation.','ColorMode.Distance.Brightness':'Distance color brightness.','ColorMode.Distance.Hue':'Distance-to-hue spline; default points (0,0), (100,120), (200,120), tension 0.4.','ColorMode.Distance.Alpha':'Distance color alpha, default 1.','ColorMode.Health.Alpha':'Health color alpha, default 255; actual health excludes absorption.','ColorMode.Static.Color':'Static ARGB color, default opaque blue-cyan.','LineWidth':'Physical line width in pixels; default 1, UI range 1–16.','MaximumDistance':'Camera distance to current entity position; default 128 blocks, UI range 1–512.','Render':'Plain lines or the original nine distance-field shader styles.','Render.Shader.Style':'Select an original post-processing style; line Fill defaults to 1.'}
def key(path):return 'liquidbounce.module.tracers'+(''.join('.'+n[0].lower()+n[1:]for n in path.split('.'))if path else'')+'.description'
def descriptions(language):
 data={key(k):v for k,v in(EN if language=='en_us'else TW if language=='zh_tw'else CN).items()}
 for k,v in BASE['descriptions'](language).items():
  if '.render.' in k:data[k.replace('.breadcrumbs.','.tracers.')]=v.replace('轨迹','射线').replace('軌跡','射線')
 for k,v in(EN if language=='en_us'else TW if language=='zh_tw'else CN).items():data[key(k)]=v
 for style in BASE['NAMES']:data[key('Render.Shader.Style.'+style+'.Fill')]='Silhouette interior opacity; Tracers defaults to 1.'if language=='en_us'else'剪影內部的填滿不透明度；射線預設為 1。'if language=='zh_tw'else'剪影内部的填充不透明度；射线默认 1。'
 return data
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));labels=BASE['LABELS_TW']if language=='zh_tw'else BASE['LABELS'];data['labels'].update({k:v for k,v in labels.items()if k not in ['Breadcrumbs','PlayerTrails','OnlyOwn','Temporary','Alive','Fade']});data['labels'].update(LABELS_TW if language=='zh_tw'else LABELS);data['descriptions'].update(descriptions(language))
 if language=='zh_cn':data['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,ROOT/a.directory/f'display_{language}.json')
