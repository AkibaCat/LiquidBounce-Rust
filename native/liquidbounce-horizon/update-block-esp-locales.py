"""Complete BlockESP Simplified/Traditional Chinese settings and style descriptions."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
BASE=runpy.run_path(str(HERE/'update-breadcrumbs-locales.py'))
TEXTS={
 'description':('Allows you to see selected blocks through walls.','让你能透过墙壁看到选定的方块。','讓你能穿透牆壁看見選定的方塊。'),
 'mode.description':('Choose box shapes or independently styled glow.','选择形状方框或独立样式的辉光。','選擇形狀方框或獨立樣式的輝光。'),
 'mode.box.outline.description':('Draw voxel-shape outlines in Box mode.','在方框模式下绘制体素形状轮廓。','在方框模式下繪製體素形狀輪廓。'),
 'mode.glow.style.description':('Choose vanilla edge detection or one of the nine post-processing styles.','选择原版边缘检测或九种后处理样式，默认为柔光。','選擇原版邊緣偵測或九種後製樣式，預設為柔光。'),
 'targets.description':('Blocks to track in loaded chunks, initially all beds and the dragon egg.','在已加载区块中追踪的方块，默认包含所有床和龙蛋。','在已載入區塊中追蹤的方塊，預設包含所有床與龍蛋。'),
 'colorMode.description':('Choose the block map color, a static color or a cycling rainbow.','使用方块地图色、固定颜色或循环彩虹色。','使用方塊地圖色、固定色彩或循環彩虹色。'),
 'colorMode.static.color.description':('ARGB color used by the static color mode.','固定颜色模式使用的 ARGB 颜色。','固定色彩模式使用的 ARGB 色彩。'),
 'distanceFade.description':('Fade block shapes near and far from the camera.','根据与相机的距离对方块形状进行近远渐隐。','根據與攝影機的距離對方塊形狀進行遠近漸隱。'),
 'distanceFade.nearStart.description':('Near fade-in start, at most NearEnd, from 0 to 512 blocks.','近端渐入起点，不大于近端终点，范围 0～512 方块。','近端漸入起點，不大於近端終點，範圍 0～512 方塊。'),
 'distanceFade.nearEnd.description':('Near fade-in end, between NearStart and FarStart.','近端渐入终点，限制在近端起点与远端起点之间。','近端漸入終點，限制在近端起點與遠端起點之間。'),
 'distanceFade.farStart.description':('Far fade-out start, between NearEnd and FarEnd.','远端渐出起点，限制在近端终点与远端终点之间。','遠端漸出起點，限制在近端終點與遠端終點之間。'),
 'distanceFade.farEnd.description':('Far fade-out end, at least FarStart.','远端渐出终点，不小于远端起点。','遠端漸出終點，不小於遠端起點。'),
 'mergeAdjacent.description':('Merge face-adjacent voxel shapes with the same block and map color, removing internal faces and edges.','合并方块类型和地图色相同且面相邻的体素形状，去除内部面和边。','合併方塊類型與地圖色相同且面相鄰的體素形狀，移除內部面與邊。'),
}
LABELS={'BlockESP':('方块透视','方塊透視'),'MapColor':('地图颜色','地圖色彩'),'ColorMode':('颜色模式','色彩模式'),'Static':('固定颜色','固定色彩'),'Rainbow':('彩虹','彩虹'),'Targets':('目标方块','目標方塊'),'Box':('方框','方框'),'Glow':('辉光','輝光'),'Vanilla':('原版','原版'),'MergeAdjacent':('合并相邻形状','合併相鄰形狀'),'DistanceFade':('距离渐隐','距離漸隱'),'NearStart':('近端起点','近端起點'),'NearEnd':('近端终点','近端終點'),'FarStart':('远端起点','遠端起點'),'FarEnd':('远端终点','遠端終點')}
def update(folder):
 for code,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{code}.json';data=json.loads(path.read_text(encoding='utf8'));texts=dict(TEXTS)
  for style,fields in zip(BASE['NAMES'],BASE['FIELDS']):
   for name in ['Opacity','Fill']+fields:
    cn=BASE['PARAMS'][name].replace('轨迹','方块');tw=BASE['PARAMS_TW'][name].replace('軌跡','方塊')
    if name=='Fill':cn='剪影内部填充不透明度；保留各原样式的独立默认值。';tw='剪影內部填滿不透明度；保留各原樣式的獨立預設值。'
    p='mode.glow.style.'+style[0].lower()+style[1:]+'.'+name[0].lower()+name[1:]+'.description';texts[p]=(f'Adjust the {name} parameter of the {style} glow style.',cn,tw)
  put_module(data,'liquidbounce.module.blockESP','BlockESP',{'':LABELS['BlockESP'][index-1],**{k:v[index]for k,v in texts.items()}},{k:v[0]for k,v in texts.items()})
  data.setdefault('labels',{}).update(BASE['LABELS_TW']if code=='zh_tw'else BASE['LABELS'])
  for k,pair in LABELS.items():data['labels'][k]=pair[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
