"""Complete Simplified/Traditional Chinese TNTTimer labels and descriptions."""
from pathlib import Path
import argparse,json,sys
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'display_locale_schema.py').is_file()else HERE.parents[1]
sys.path.insert(0,str(NATIVE));from display_locale_schema import put_module
LABELS={'TNTTimer':('TNT 倒计时','TNT 倒數計時'),'ESP':('发光轮廓','發光輪廓'),'ShowTimer':('显示倒计时','顯示倒數計時'),'Enabled':('启用','啟用'),'Scale':('缩放曲线','縮放曲線'),'RenderY':('垂直偏移','垂直偏移'),'OwnerName':('显示引燃者','顯示引燃者'),'TimeUnit':('时间单位','時間單位'),'Show':('显示位置','顯示位置'),'Ticks':('游戏刻','遊戲刻'),'Seconds':('秒','秒'),'InView':('屏幕内','螢幕內'),'Offscreen':('屏幕外','螢幕外'),'Distance':('距离','距離')}
TEXTS={
'description':('Highlight primed TNT and display its fuse with indicators.','高亮已引燃的 TNT，并用指示器显示引信倒计时。','醒目標示已引燃的 TNT，並用指示器顯示引信倒數。'),
'esp.description':('Outline actual TNT geometry in a red pulse that accelerates with the fuse.','为 TNT 的实际模型绘制红色脉冲发光轮廓，引信越短，变化越快。','為 TNT 的實際模型繪製紅色脈衝發光輪廓，引信越短，變化越快。'),
'showTimer.description':('Display each live TNT fuse, with an arrow when it is offscreen.','显示每个有效 TNT 引信；屏幕外的 TNT 使用方向箭头。','顯示每個有效 TNT 引信；螢幕外的 TNT 使用方向箭頭。'),
'showTimer.enabled.description':('Enable TNT fuse labels and offscreen indicators.','启用 TNT 引信标签及屏外指示器。','啟用 TNT 引信標籤及螢幕外指示器。'),
'showTimer.scale.description':('Scale the countdown with the original distance spline (0 to 200 blocks; scale 0.25 to 4).','按距离曲线缩放倒计时：横轴为 0～200 格距离，纵轴为 0.25～4 倍缩放，使用原始样条。','依距離曲線縮放倒數計時：橫軸為 0～200 格距離，縱軸為 0.25～4 倍縮放，使用原始樣條。'),
'showTimer.renderY.description':('Vertical offset from the current TNT bounding-box center.','相对于 TNT 当前边界框中心的垂直偏移。','相對於 TNT 目前邊界框中心的垂直偏移。'),
'showTimer.ownerName.description':('Append the TNT owner name when the client knows its living owner.','客户端知道 TNT 的生物引燃者时，在倒计时后显示其名称。','用戶端知道 TNT 的生物引燃者時，在倒數計時後顯示其名稱。'),
'showTimer.timeUnit.description':('Display the remaining fuse in ticks or seconds with up to one decimal.','以游戏刻或最多一位小数的秒显示剩余引信。','以遊戲刻或最多一位小數的秒顯示剩餘引信。'),
'showTimer.show.description':('Choose in-view and offscreen indicators; at least one location is required.','选择屏幕内或屏幕外的指示器，至少保留一种显示位置。','選擇螢幕內或螢幕外的指示器，至少保留一種顯示位置。')}
def update(lang,path):
 index=1 if lang=='zh_cn'else 2;data=json.loads(path.read_text(encoding='utf8'));entries={'':LABELS['TNTTimer'][index-1],**{k:v[index]for k,v in TEXTS.items()}}
 put_module(data,'liquidbounce.module.tntTimer','TNTTimer',entries,{k:v[0]for k,v in TEXTS.items()});data['labels'].update({k:v[index-1]for k,v in LABELS.items()});path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',type=Path,default=NATIVE.parent/'liquidbounce-interop/resources/localization');args=a.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,args.directory/f'display_{lang}.json')
