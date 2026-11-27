"""BlinkManager globals in both nested Chinese display dictionaries, explicit directory."""
from pathlib import Path
import argparse,json
LABELS={'BlinkManager':('Blink 管理器','Blink 管理器'),'Esp':('位置显示','位置顯示'),'Box':('方框','方框'),'Model':('玩家模型','玩家模型'),'Wireframe':('线框玩家','線框玩家'),'Ring':('圆环','圓環'),'None':('无','無'),'Line':('轨迹线颜色','軌跡線顏色'),'LightPercent':('模型亮度','模型亮度')}
DESCRIPTIONS={
'':('配置共享 Blink 队列的全局位置显示与轨迹线。','設定共用 Blink 佇列的全域位置顯示與軌跡線。'),
'Esp':('选择队列首个移动位置的显示方式；第一视角隐藏方框、模型和线框。','選擇佇列第一個移動位置的顯示方式；第一人稱隱藏方框、模型與線框。'),
'Esp.Box':('按玩家当前姿态的实际尺寸显示方框，并向外扩展 0.05 格。','依玩家目前姿態的實際尺寸顯示方框，並向外擴展 0.05 格。'),
'Esp.Model':('使用玩家真实皮肤、装备和姿态，按服务端实际朝向显示。','使用玩家真實外觀、裝備與姿態，依伺服器實際朝向顯示。'),
'Esp.Model.LightPercent':('分别缩放模型的天空光和方块光，0 至 100%。','分別縮放模型的天空光與方塊光，0 至 100%。'),
'Esp.Wireframe':('按当前站立、潜行或游泳姿态及服务端实际朝向显示六部分线框。','依目前站立、蹲伏或游泳姿態及伺服器實際朝向顯示六部分線框。'),
'Esp.Ring':('在首个移动位置绘制中心透明、边缘着色的地面圆环；第一视角仍显示。','在第一個移動位置繪製中心透明、邊緣著色的地面圓環；第一人稱仍顯示。'),
'Esp.Ring.Radius':('设置地面圆环的半径，单位为格。','設定地面圓環半徑，單位為格。'),
'Esp.None':('关闭位置形状；轨迹线仍可单独显示。','關閉位置形狀；軌跡線仍可單獨顯示。'),
'Line':('按缓存顺序连接所有包含位置的移动包；透明度为 0 时隐藏。','依快取順序連接所有包含位置的移動封包；透明度為 0 時隱藏。'),
'Render':('选择轨迹线的普通或 Shader 渲染方式，所有样式的默认填充为 1。','選擇軌跡線的普通或 Shader 算繪方式，所有樣式的預設填充為 1。')}
def key(path):return 'liquidbounce.settings.blinkManager'+('.'+'.'.join(x[:1].lower()+x[1:]for x in path.split('.'))if path else'')+'.description'
def update(language,path):
 d=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 for k,v in LABELS.items():d.setdefault('labels',{})[k]=v[i]
 descriptions=d.setdefault('descriptions',{});english=d.setdefault('englishDescriptions',{})
 for k,v in list(descriptions.items()):
  if k.startswith('liquidbounce.module.breadcrumbs.render'):descriptions[k.replace('liquidbounce.module.breadcrumbs','liquidbounce.settings.blinkManager')]=v
 for k,v in list(english.items()):
  if k.startswith('liquidbounce.module.breadcrumbs.render'):english[k.replace('liquidbounce.module.breadcrumbs','liquidbounce.settings.blinkManager')]=v
 fields=['','Esp','Line','Render']
 for mode in ['Box','Model','Wireframe','Ring','None']:
  fields.append('Esp.'+mode)
  fields +=['Esp.'+mode+'.'+name for name in ({'Box':['Color','OutlineColor'],'Model':['OutlineColor','LightPercent'],'Wireframe':['Color','OutlineColor'],'Ring':['Radius','Color','OutlineColor'],'None':[]}[mode])]
 for p in fields:
  name=p.split('.')[-1];fallback=('设置填充颜色。','設定填滿顏色。')if name=='Color'else('设置轮廓颜色；透明度为 0 时隐藏轮廓。','設定輪廓顏色；透明度為 0 時隱藏輪廓。')
  descriptions[key(p)]=DESCRIPTIONS.get(p,fallback)[i];english[key(p)]=f'BlinkManager {p or "global"} setting.'
 for k in list(d):
  if k.startswith('liquidbounce.settings.blinkManager'):del d[k]
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
