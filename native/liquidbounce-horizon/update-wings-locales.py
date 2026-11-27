"""Complete Wings simplified/traditional Chinese keys; caller selects dictionary directory."""
from pathlib import Path
import argparse,json
LABELS={'':('翅膀','翅膀'),'Position':('位置','位置'),'Position.Height':('高度','高度'),'Position.BackOffset':('背后偏移','背後偏移'),'Position.EquipmentOffset':('胸甲偏移','胸甲偏移'),'Mode':('模式','模式'),'Mode.Lines':('线片','線片'),'Mode.Lines.ShowDamage':('显示受伤','顯示受傷'),'Mode.Lines.ShowInFirstPerson':('第一人称显示','第一人稱顯示'),'Mode.Lines.FriendsOptions':('好友选项','好友選項'),'Mode.Lines.FriendsOptions.ViewOnFriend':('显示好友翅膀','顯示好友翅膀'),'Mode.Lines.FriendsOptions.Distance':('距离','距離'),'Mode.Lines.Length':('长度','長度'),'Mode.Lines.Width':('宽度','寬度'),'Mode.Lines.FadeStartRatio':('渐隐起点','漸隱起點'),'Mode.Lines.LinesCount':('线片数量','線片數量'),'Mode.Lines.Angles':('角度范围','角度範圍'),'Mode.Lines.Color':('颜色','顏色')}
DESC={'':('为自己和好友绘制跟随身体朝向的线片翅膀。','為自己與好友繪製跟隨身體朝向的線片翅膀。'),'Position.Height':('设置翅膀相对玩家包围盒中部的高度。','設定翅膀相對玩家邊界框中部的高度。'),'Position.BackOffset':('设置翅膀向玩家背后移动的距离，随实体体型缩放。','設定翅膀向玩家背後移動的距離，隨實體體型縮放。'),'Position.EquipmentOffset':('胸部装备非空时，额外向背后移动的距离。','胸部裝備非空時，額外向背後移動的距離。'),'Mode.Lines.ShowDamage':('受伤时显示红色翅膀并保留配置的透明度。','受傷時顯示紅色翅膀並保留設定的透明度。'),'Mode.Lines.ShowInFirstPerson':('在第一人称显示自己的翅膀；启用自由视角时也会显示。','在第一人稱顯示自己的翅膀；啟用自由視角時也會顯示。'),'Mode.Lines.FriendsOptions.ViewOnFriend':('为好友列表中的其他玩家显示翅膀。','為好友清單中的其他玩家顯示翅膀。'),'Mode.Lines.FriendsOptions.Distance':('仅为三维距离严格小于此值的好友显示翅膀。','僅為三維距離嚴格小於此值的好友顯示翅膀。'),'Mode.Lines.Length':('设置线片长度，保留原版两侧不同的四分之一格长度差。','設定線片長度，保留原版兩側不同的四分之一格長度差。'),'Mode.Lines.Width':('设置每条线片的宽度。','設定每條線片的寬度。'),'Mode.Lines.FadeStartRatio':('设置从线片根部到末端开始变透明的位置比例。','設定從線片根部至末端開始變透明的位置比例。'),'Mode.Lines.LinesCount':('设置每侧展开的线片数量。','設定每側展開的線片數量。'),'Mode.Lines.Angles':('将线片角度均匀分布在范围内，只有一片时使用起始角度。','將線片角度均勻分布於範圍內，只有一片時使用起始角度。'),'Mode.Lines.Color':('设置翅膀颜色与透明度，Shader模式使用不透明轮廓。','設定翅膀顏色與透明度，Shader模式使用不透明輪廓。')}
def lower(p):return'.'.join(s[:1].lower()+s[1:]for s in p.split('.'))
def update(language,path):
 data=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 data.setdefault('moduleNames',{})['Wings']=LABELS[''][i]
 for key,label in LABELS.items():
  if key:data.setdefault('labels',{})[key.split('.')[-1]]=label[i]
  p='liquidbounce.module.wings'+('.'+lower(key)if key else'');data.setdefault('descriptions',{})[p+'.description']=DESC.get(key,(f'设置{label[0]}。',f'設定{label[1]}。'))[i]
 for section in ['descriptions','englishDescriptions']:
  for key,value in list(data.setdefault(section,{}).items()):
   if key.startswith('liquidbounce.module.breadcrumbs.render'):
    value=value.replace('轨迹','翅膀').replace('軌跡','翅膀').replace('trail','wing').replace('Trail','Wing')
    if key.endswith('.fill.description'):value='Silhouette interior opacity; Wings defaults to 1.'if section=='englishDescriptions'else'剪影內部的填滿不透明度；翅膀預設為 1。'if i else'剪影内部的填充不透明度；翅膀默认为 1。'
    data[section][key.replace('liquidbounce.module.breadcrumbs','liquidbounce.module.wings.mode.lines',1)]=value
 for key in list(data):
  if key.startswith('liquidbounce.module.wings'):del data[key]
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
