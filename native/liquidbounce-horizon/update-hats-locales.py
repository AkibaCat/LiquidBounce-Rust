"""Full Hats simplified/traditional Chinese additions; caller chooses canonical or copied files."""
from pathlib import Path
import argparse,json
MODES=['Cone','Halo','Orbs','Flower','Star','Image']
CN={'':'帽子','description':'为自己和好友绘制可自定义形状、颜色与动画的帽子。','mode':'模式','mode.description':'选择圆锥、光环、环绕球、花朵、星形或图片帽子。','heightOffset':'高度偏移','heightOffset.description':'按客户端运行时间采样曲线，为帽子添加高度动画。','heightOffset.period':'周期','heightOffset.period.description':'高度动画从曲线起点运行到终点所需的时间。','heightOffset.symmetric':'往返播放','heightOffset.symmetric.description':'到达曲线终点后反向播放，形成连续的往返动画。','heightOffset.height':'高度','heightOffset.height.description':'横轴为动画进度，纵轴为帽子高于头顶的偏移。'}
TW={'':'帽子','description':'為自己和好友繪製可自訂形狀、顏色與動畫的帽子。','mode':'模式','mode.description':'選擇圓錐、光環、環繞球、花朵、星形或圖片帽子。','heightOffset':'高度偏移','heightOffset.description':'按用戶端執行時間取樣曲線，為帽子加入高度動畫。','heightOffset.period':'週期','heightOffset.period.description':'高度動畫從曲線起點執行至終點所需的時間。','heightOffset.symmetric':'往返播放','heightOffset.symmetric.description':'到達曲線終點後反向播放，形成連續的往返動畫。','heightOffset.height':'高度','heightOffset.height.description':'橫軸為動畫進度，縱軸為帽子高於頭頂的偏移。'}
LABELS={
'Cone':('圆锥','圓錐'),'Halo':('光环','光環'),'Orbs':('环绕球','環繞球'),'Flower':('花朵','花朵'),'Star':('星形','星形'),'Image':('图片','圖片'),
'FollowRotation':('跟随视角旋转','跟隨視角旋轉'),'EquipmentOffset':('装备偏移','裝備偏移'),'ArmorOffset':('头盔偏移','頭盔偏移'),'ShowDamage':('显示受伤','顯示受傷'),'FriendsOptions':('好友选项','好友選項'),'ViewOnFriend':('显示好友帽子','顯示好友帽子'),'Distance':('距离','距離'),'FirstPersonView':('第一人称显示','第一人稱顯示'),'HatSettings':('帽子设置','帽子設定'),'RadiusSettings':('半径设置','半徑設定'),'Peak':('锥顶高度','錐頂高度'),'OuterRadius':('外半径','外半徑'),'Radius':('半径','半徑'),'Thickness':('厚度','厚度'),'Sharpness':('尖锐度','尖銳度'),'PetalCount':('花瓣数量','花瓣數量'),'PointsCount':('星角数量','星角數量'),'Speed':('公转速度','公轉速度'),'OrbsSize':('球体大小','球體大小'),'OrbsCount':('球体数量','球體數量'),'SpinSpeed':('旋转速度','旋轉速度'),'Wave':('上下波动','上下波動'),'Enabled':('启用','啟用'),'WaveHeight':('波动高度','波動高度'),'WaveSpeed':('波动速度','波動速度'),'Colors':('颜色','顏色'),'SyncColors':('同步颜色','同步顏色'),'FirstColor':('第一颜色','第一顏色'),'SecondColor':('第二颜色','第二顏色'),'color':('颜色','顏色'),'ColorModulator':('图片着色','圖片著色'),'Scale':('缩放','縮放')}
DESC={
'FollowRotation':('使帽子跟随实体的偏航和俯仰角旋转。','使帽子跟隨實體的偏航與俯仰角旋轉。'),
'ArmorOffset':('佩戴头盔或其他头部物品时额外抬高帽子的距离。','佩戴頭盔或其他頭部物品時額外抬高帽子的距離。'),
'ShowDamage':('实体受伤时将几何帽子标为红色，并保留配置的透明度。图片模式保留图片着色。','實體受傷時將幾何帽子標為紅色，並保留設定的透明度。圖片模式保留圖片著色。'),
'ViewOnFriend':('为好友列表中的其他玩家显示帽子。','為好友清單中的其他玩家顯示帽子。'),
'Distance':('仅为三维距离严格小于此值的好友显示帽子。','僅為三維距離嚴格小於此值的好友顯示帽子。'),
'FirstPersonView':('允许第一人称显示自己的帽子；启用自由视角时也会显示。','允許第一人稱顯示自己的帽子；啟用自由視角時也會顯示。'),
'SyncColors':('所有分段使用第一颜色；关闭后在两种颜色间循环混合。','所有分段使用第一顏色；關閉後在兩種顏色間循環混合。'),
'FirstColor':('帽子的第一颜色及受伤标记的透明度。','帽子的第一顏色與受傷標記的透明度。'),
'SecondColor':('关闭颜色同步时使用的第二种渐变颜色。','關閉顏色同步時使用的第二種漸層顏色。'),
'SpinSpeed':('设置旋转速度；负值反向旋转。颜色组的速度控制渐变流动。','設定旋轉速度；負值反向旋轉。顏色群組的速度控制漸層流動。'),
'Image':('选择 PNG 或 JPEG 图片作为水平帽子。加载失败时保留上一张有效纹理。','選擇 PNG 或 JPEG 圖片作為水平帽子。載入失敗時保留上一張有效紋理。'),
'Scale':('分别设置图片横向和纵向的缩放比例。','分別設定圖片橫向與縱向的縮放比例。'),
'ColorModulator':('将颜色与图片像素相乘，包含透明度。图片帽子始终使用普通渲染。','將顏色與圖片像素相乘，包含透明度。圖片帽子一律使用一般繪製。'),
'Wave':('让环绕球按正弦曲线上下移动。','讓環繞球按正弦曲線上下移動。'),
'WaveHeight':('环绕球上下移动的最大高度。','環繞球上下移動的最大高度。'),
'WaveSpeed':('环绕球上下波动的速度。','環繞球上下波動的速度。')}
COMMON=['FollowRotation','EquipmentOffset','EquipmentOffset.ArmorOffset','ShowDamage','FriendsOptions','FriendsOptions.ViewOnFriend','FriendsOptions.Distance','FirstPersonView']
SPECIAL=[['HatSettings','HatSettings.Peak','RadiusSettings','RadiusSettings.OuterRadius'],['HatSettings','HatSettings.Radius','HatSettings.Thickness'],['color','HatSettings','HatSettings.Radius','HatSettings.Speed','HatSettings.OrbsSize','HatSettings.OrbsCount','HatSettings.SpinSpeed','Wave','Wave.Enabled','Wave.WaveHeight','Wave.WaveSpeed'],['HatSettings','HatSettings.Radius','HatSettings.Thickness','HatSettings.Sharpness','HatSettings.PetalCount','HatSettings.SpinSpeed'],['HatSettings','HatSettings.Radius','HatSettings.Thickness','HatSettings.Sharpness','HatSettings.PointsCount','HatSettings.SpinSpeed'],['Image','ColorModulator','Scale','SpinSpeed']]
def lower(path):return'.'.join(p[:1].lower()+p[1:]for p in path.split('.'))
def update(language,path):
 data=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1;translations=dict(CN if i==0 else TW)
 for n,name in enumerate(MODES):
  prefix='mode.'+lower(name);translations[prefix]=LABELS[name][i];translations[prefix+'.description']=('绘制'+LABELS[name][i]+'帽子。')if i==0 else('繪製'+LABELS[name][i]+'帽子。')
  fields=COMMON+SPECIAL[n]+(['Colors','Colors.SyncColors','Colors.FirstColor','Colors.SecondColor','Colors.SpinSpeed']if n in[0,1,3,4]else[])
  for field in fields:
   tail=field.split('.')[-1];key=prefix+'.'+lower(field);translations[key]=LABELS[tail][i];translations[key+'.description']=DESC.get(tail,(f'设置{LABELS[tail][0]}。',f'設定{LABELS[tail][1]}。'))[i]
 data.setdefault('moduleNames',{})['Hats']=('帽子','帽子')[i]
 data.setdefault('labels',{}).update({name:label[i]for name,label in LABELS.items()})
 data['labels'].update({'HeightOffset':'高度偏移','Period':'週期'if i else'周期','Symmetric':'往返播放','Hats':'帽子'})
 for section in ['descriptions','englishDescriptions']:
  for k,v in list(data.setdefault(section,{}).items()):
   if k.startswith('liquidbounce.module.breadcrumbs.render'):data[section][k.replace('liquidbounce.module.breadcrumbs','liquidbounce.module.hats',1)]=v.replace('轨迹','帽子').replace('軌跡','帽子').replace('trail','hat').replace('Trail','Hat')
 for k,v in translations.items():
  if k=='description'or k.endswith('.description'):data['descriptions']['liquidbounce.module.hats.'+k]=v
 for k in list(data):
  if k.startswith('liquidbounce.module.hats'):del data[k]
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
