"""Complete nested Simplified/Traditional Chinese Particles dictionaries."""
from pathlib import Path
import argparse,json
DESCRIPTIONS={
'':('攻击可见目标时生成带有方块碰撞和弹跳的纹理粒子。','攻擊可見目標時產生具有方塊碰撞和彈跳的紋理粒子。'),
'Size':('设置攻击粒子的初始大小；随后随生成时间缩小。','設定攻擊粒子的初始大小；隨後依生成時間縮小。'),
'Count':('设置每次有效攻击随机生成的粒子数量范围。','設定每次有效攻擊隨機產生的粒子數量範圍。'),
'RandomParticleRotation':('使用每个粒子随机的纹理旋转角度。','使用每個粒子隨機的紋理旋轉角度。'),
'Physical':('配置粒子的水平运动、方块碰撞弹跳和重力。','設定粒子的水平運動、方塊碰撞彈跳和重力。'),
'Physical.Motion':('设置水平速度倍率；不改变竖直速度。','設定水平速度倍率；不改變垂直速度。'),
'Physical.BounceX':('设置 X 轴碰撞后的反向速度比例。','設定 X 軸碰撞後的反向速度比例。'),
'Physical.BounceY':('设置竖直碰撞后的反向速度比例。','設定垂直碰撞後的反向速度比例。'),
'Physical.BounceZ':('设置 Z 轴碰撞后的反向速度比例。','設定 Z 軸碰撞後的反向速度比例。'),
'Physical.Drag':('设置竖直碰撞后保留的水平速度比例。','設定垂直碰撞後保留的水平速度比例。'),
'Physical.GravityFactor':('设置每刻施加的重力比例。','設定每刻施加的重力比例。'),
'Color':('设置粒子的颜色与初始透明度；首次碰撞后持续三秒淡出。','設定粒子的顏色與初始透明度；首次碰撞後持續三秒淡出。'),
'Particle':('选择随机使用的内置纹理，至少保留一种。','選擇隨機使用的內建紋理，至少保留一種。'),
'Bind':('设置切换攻击粒子的按键。','設定切換攻擊粒子的按鍵。'),
'Hidden':('在模块列表中隐藏攻击粒子。','在模組清單中隱藏攻擊粒子。')}
LABELS={'Particles':('攻击粒子','攻擊粒子'),'RandomParticleRotation':('随机纹理旋转','隨機紋理旋轉'),'Physical':('物理效果','物理效果'),'Motion':('水平速度倍率','水平速度倍率'),'BounceX':('X 轴弹跳','X 軸彈跳'),'BounceY':('Y 轴弹跳','Y 軸彈跳'),'BounceZ':('Z 轴弹跳','Z 軸彈跳'),'GravityFactor':('重力比例','重力比例'),'Orbiz':('光球','光球'),'Star':('星星','星星'),'Dollar':('美元','美元'),'Crown':('皇冠','皇冠'),'Heart':('爱心','愛心'),'Lightning':('闪电','閃電'),'Line':('线条','線條'),'Point':('光点','光點'),'Rhombus':('菱形','菱形'),'Snowflake':('雪花','雪花'),'Spark':('火花','火花')}
def key(path):return 'liquidbounce.module.particles'+('.'+'.'.join(s[:1].lower()+s[1:]for s in path.split('.'))if path else'')+'.description'
def update(language,path):
 d=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 for label,translations in LABELS.items():d.setdefault('labels',{})[label]=translations[i]
 for path_name,translations in DESCRIPTIONS.items():d.setdefault('descriptions',{})[key(path_name)]=translations[i];d.setdefault('englishDescriptions',{})[key(path_name)]=f'Particles {path_name or "module"} setting.'
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
