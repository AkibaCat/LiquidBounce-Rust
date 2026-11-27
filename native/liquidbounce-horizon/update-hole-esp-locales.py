"""HoleESP nested Chinese dictionaries; caller chooses copies or canonical directory."""
from pathlib import Path
import argparse,json
DESCRIPTIONS={
'':('检测并标记有三格净空和抗爆边界的安全坑。','偵測並標記具有三格淨空和抗爆邊界的安全坑。'),
'Mode':('选择完整方框或带有渐变光柱的地面平面。','選擇完整方框或附有漸層光柱的地面平面。'),
'Mode.Box':('以一格高的完整方框标记洞位。','以一格高的完整方框標記洞位。'),
'Mode.Box.Outline':('显示完整方框的边线。','顯示完整方框的邊線。'),
'Mode.GlowingPlane':('绘制底面及四面逐渐透明的光柱。','繪製底面及四面逐漸透明的光柱。'),
'Mode.GlowingPlane.Outline':('显示底面周围的边线。','顯示底面周圍的邊線。'),
'Mode.GlowingPlane.GlowHeight':('设置渐变光柱高度，单位为格。','設定漸層光柱高度，單位為格。'),
'HorizontalScanDistance':('设置洞位扫描的水平范围；移动四格后更新扫描区域。','設定洞位掃描的水平範圍；移動四格後更新掃描區域。'),
'VerticalScanDistance':('设置玩家上下的洞位扫描范围。','設定玩家上下的洞位掃描範圍。'),
'DistanceFade':('控制扫描范围边缘的透明渐变；0 表示不淡出。','控制掃描範圍邊緣的透明漸層；0 表示不淡出。'),
'1x1Bedrock':('设置四壁和底部均为不可破坏方块的单格坑颜色。','設定四壁和底部皆為不可破壞方塊的單格坑顏色。'),
'1x1':('设置具有抗爆四壁和底部的普通单格坑颜色。','設定具有抗爆四壁和底部的一般單格坑顏色。'),
'1x2':('设置双格安全坑的颜色。','設定雙格安全坑的顏色。'),
'2x2':('设置四格安全坑的颜色。','設定四格安全坑的顏色。'),
'Bind':('设置切换安全坑透视的按键。','設定切換安全坑透視的按鍵。'),
'Hidden':('在模块列表中隐藏安全坑透视。','在模組清單中隱藏安全坑透視。')}
def key(path):return 'liquidbounce.module.holeESP'+('.'+'.'.join(s[:1].lower()+s[1:]for s in path.split('.'))if path else'')+'.description'
def update(language,path):
 d=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 for label,translations in {'HoleESP':('安全坑透视','安全坑透視'),'GlowingPlane':('渐变光柱','漸層光柱'),'GlowHeight':('光柱高度','光柱高度'),'1x1Bedrock':('基岩单格坑','基岩單格坑'),'1x1':('普通单格坑','一般單格坑'),'1x2':('双格坑','雙格坑'),'2x2':('四格坑','四格坑')}.items():d.setdefault('labels',{})[label]=translations[i]
 for p,value in DESCRIPTIONS.items():d.setdefault('descriptions',{})[key(p)]=value[i];d.setdefault('englishDescriptions',{})[key(p)]=f'HoleESP {p or "module"} setting.'
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
