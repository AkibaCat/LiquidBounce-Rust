from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'PlaceInLiquid':('在液体中放置','在液體中放置'),'CustomRange':('自定义距离','自訂距離'),'ScrollAdjust':('滚轮调节','滾輪調節'),'OutlineColor':('轮廓颜色','輪廓顏色')}
TEXTS={
'':('Place supported items at the aimed air position.','在准星指向的空气位置放置可用物品。','在準星指向的空氣位置放置可用物品。'),
'placeInLiquid':('Allow fluid targets while LiquidPlace is inactive.','允许在液体位置放置；液体放置模块运行时暂停此项。','允許在液體位置放置；液體放置模組執行時暫停此項。'),
'preview.enabled':('Preview the current valid target.','预览当前有效的放置位置。','預覽目前有效的放置位置。'),
'preview.outlineOnly':('Draw only the target outline.','仅绘制放置位置的轮廓。','僅繪製放置位置的輪廓。'),
'preview.color':('Preview fill color and opacity.','预览填充颜色及透明度。','預覽填滿顏色與透明度。'),
'preview.outlineColor':('Preview outline color and opacity.','预览轮廓颜色及透明度。','預覽輪廓顏色與透明度。'),
'customRange.enabled':('Move the target along the original sight line to a custom distance.','沿原瞄准方向将目标移至自定义距离。','沿原瞄準方向將目標移至自訂距離。'),
'customRange.range':('Distance from the player eye to the target.','玩家眼睛到放置目标的距离。','玩家眼睛到放置目標的距離。'),
'customRange.scrollAdjust.enabled':('Adjust the range with the mouse wheel.','使用鼠标滚轮调节距离。','使用滑鼠滾輪調節距離。'),
'customRange.scrollAdjust.modifier':('Hold this key while scrolling; an unbound key requires no modifier.','滚动时按住此键；未绑定时无需修饰键。','滾動時按住此鍵；未綁定時無需修飾鍵。'),
'customRange.scrollAdjust.sensitivity':('Range change per wheel step, clamped to 1–4.5 blocks.','每次滚动的距离变化，滚轮调节范围限制为 1～4.5 格。','每次滾動的距離變化，滾輪調節範圍限制為 1～4.5 格。')}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 p=FOLDER/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));d['moduleNames']['AirPlace']=('空中放置','空中放置')[index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
 for suffix,text in TEXTS.items():
  key='liquidbounce.module.airPlace'+('.'+suffix if suffix else '')+'.description';d['descriptions'][key]=text[index]
  if locale=='zh_cn':d['englishDescriptions'][key]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'AirPlace help entries')
