"""Complete pinned Crosshair Chinese display dictionaries; explicit directory only."""
from pathlib import Path
import argparse,json
LABELS={'Crosshair':('准星','準星'),'Circle':('圆环','圓環'),'CS2':('CS2','CS2'),'Radius':('半径','半徑'),'Range':('范围','範圍'),'ShowInThirdPerson':('第三人称显示','第三人稱顯示'),'DynamicRadiusMultiplier':('动态半径倍率','動態半徑倍率'),'Length':('长度','長度'),'Thickness':('粗细','粗細'),'Gap':('间距','間距'),'DynamicMultiplier':('动态倍率','動態倍率'),'Sync':('同步颜色','同步顏色'),'FirstColor':('第一颜色','第一顏色'),'SecondColor':('第二颜色','第二顏色'),'SpinSpeed':('颜色转速','顏色轉速'),'CooldownBar':('冷却条','冷卻條'),'Sources':('来源','來源'),'Charge':('蓄力','蓄力'),'Item':('物品','物品'),'Attack':('攻击','攻擊'),'Width':('宽度','寬度'),'Height':('高度','高度'),'Offset':('偏移','偏移'),'Color':('颜色','顏色'),'Background':('背景','背景'),'ReadyFlash':('就绪闪色','就緒閃色'),'Duration':('持续时间','持續時間'),'HitFlash':('受击闪色','受擊閃色'),'Enabled':('启用','啟用'),'Mode':('模式','模式')}
def paths():
 out=['','Mode','CooldownBar','CooldownBar.Enabled','CooldownBar.Sources','CooldownBar.Width','CooldownBar.Height','CooldownBar.Offset','CooldownBar.Color','CooldownBar.Background','CooldownBar.ReadyFlash','CooldownBar.ReadyFlash.Enabled','CooldownBar.ReadyFlash.Color','CooldownBar.ReadyFlash.Duration','HitFlash','HitFlash.Enabled','HitFlash.Color','HitFlash.Duration']
 for mode in ['Circle','CS2']:
  prefix=f'Mode.{mode}';out +=[prefix,prefix+'.ShowInThirdPerson',prefix+'.Color']+[prefix+'.Color.'+f for f in ['Sync','FirstColor','SecondColor','SpinSpeed']]
  group='Radius'if mode=='Circle'else'Crosshair';out +=[prefix+'.'+group]+[prefix+'.'+group+'.'+f for f in (['Range','DynamicRadiusMultiplier']if mode=='Circle'else['Length','Thickness','Gap','DynamicMultiplier'])]
 return out
SPEC={'':('替换默认准星，提供圆环、CS2、冷却条和受击闪色。','替換預設準星，提供圓環、CS2、冷卻條與受擊閃色。'),'CooldownBar.Sources':('按蓄力、物品、攻击的固定优先级，显示第一个尚未就绪的来源。','依蓄力、物品、攻擊的固定優先順序，顯示第一個尚未就緒的來源。'),'CooldownBar.ReadyFlash':('来源从未就绪变为就绪时闪色；提前松开蓄力不会闪色。','來源從未就緒變為就緒時閃色；提前放開蓄力不會閃色。'),'HitFlash':('持续瞄准的同一生物受击计时增加时，为准星短暂叠加颜色。','持續瞄準的同一生物受擊計時增加時，為準星短暫疊加顏色。')}
def update(language,path):
 d=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 d.setdefault('moduleNames',{})['Crosshair']=LABELS['Crosshair'][i]
 for key,value in LABELS.items():d.setdefault('labels',{})[key]=value[i]
 for p in paths():
  k='liquidbounce.module.crosshair'+('.'+'.'.join(s[:1].lower()+s[1:]for s in p.split('.'))if p else'')+'.description'
  n=p.split('.')[-1]if p else'Crosshair';label=LABELS.get(n,(n,n))[i]
  if p in SPEC:description=SPEC[p][i]
  elif n in ['DynamicMultiplier','DynamicRadiusMultiplier']:description=('按攻击冷却未恢复的比例扩大准星；设为 0 可关闭动态效果。'if i==0 else'依攻擊冷卻尚未恢復的比例擴大準星；設為 0 可關閉動態效果。')
  elif n=='ShowInThirdPerson':description='在第三人称视角显示此模式的准星。'if i==0 else'在第三人稱視角顯示此模式的準星。'
  elif n=='SpinSpeed':description='设置两种颜色随时间旋转的速度，负值反向，0 停止。'if i==0 else'設定兩種顏色隨時間旋轉的速度，負值反向，0 停止。'
  elif n=='Sync':description='启用后第一颜色用于整个准星。'if i==0 else'啟用後第一顏色用於整個準星。'
  else:description=('设置'if i==0 else'設定')+label+'。'
  d.setdefault('descriptions',{})[k]=description
  d.setdefault('englishDescriptions',{})[k]=f'Crosshair {p or "module"} setting.'
 for k in list(d):
  if k.startswith('liquidbounce.module.crosshair'):del d[k]
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
