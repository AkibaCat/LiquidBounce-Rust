#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
CN={'':'生命值变化粒子','description':'在实体上方显示伤害和治疗数值。','timeToLive':'存在时间','timeToLive.description':'粒子保留的秒数。','scale':'缩放','scale.description':'文字的目标缩放倍数。','scaleTransition':'缩放缓动','scaleTransition.description':'粒子存在期间的文字缩放变化曲线。','displacement':'位移','displacement.description':'粒子在整个生命周期内移动的世界坐标距离。','displacementTransition':'位移缓动','displacementTransition.description':'粒子存在期间的位移变化曲线。','trackMode':'追踪模式','trackMode.description':'逐刻比较生命值，或在调用生命值设置器时捕获变化。','trackMode.choice.onTick':'逐刻','trackMode.choice.onUpdate':'更新时','colors':'颜色','colors.description':'伤害、死亡和治疗粒子的文字颜色。','colors.damage':'伤害','colors.damage.description':'生命值降低时的粒子颜色。','colors.death':'死亡','colors.death.description':'生命值归零时的粒子颜色。','colors.heal':'治疗','colors.heal.description':'获得治疗但未满血时的粒子颜色。','colors.maxHealth':'满血','colors.maxHealth.description':'治疗后达到最大生命值时的粒子颜色。'}
TW={'':'生命值變化粒子','description':'在實體上方顯示傷害和治療數值。','timeToLive':'存在時間','timeToLive.description':'粒子保留的秒數。','scale':'縮放','scale.description':'文字的目標縮放倍數。','scaleTransition':'縮放緩動','scaleTransition.description':'粒子存在期間的文字縮放變化曲線。','displacement':'位移','displacement.description':'粒子在整個生命週期內移動的世界座標距離。','displacementTransition':'位移緩動','displacementTransition.description':'粒子存在期間的位移變化曲線。','trackMode':'追蹤模式','trackMode.description':'逐刻比較生命值，或在呼叫生命值設定器時擷取變化。','trackMode.choice.onTick':'逐刻','trackMode.choice.onUpdate':'更新時','colors':'顏色','colors.description':'傷害、死亡和治療粒子的文字顏色。','colors.damage':'傷害','colors.damage.description':'生命值降低時的粒子顏色。','colors.death':'死亡','colors.death.description':'生命值歸零時的粒子顏色。','colors.heal':'治療','colors.heal.description':'獲得治療但未滿血時的粒子顏色。','colors.maxHealth':'滿血','colors.maxHealth.description':'治療後達到最大生命值時的粒子顏色。'}
EN={'description':'Display damage and healing values above entities.',
 'timeToLive.description':'Particle lifetime, measured in seconds.',
 'scale.description':'Target scale of the particle text.',
 'scaleTransition.description':'Text-scale easing curve over the particle lifetime.',
 'displacement.description':'World-coordinate displacement over the particle lifetime.',
 'displacementTransition.description':'Displacement easing curve over the particle lifetime.',
 'trackMode.description':'Compare health every tick or capture changes when the health setter is called.',
 'colors.description':'Text colors for damage, death, healing and full-health particles.',
 'colors.damage.description':'Particle color when health decreases.',
 'colors.death.description':'Particle color when health reaches zero.',
 'colors.heal.description':'Particle color when healing leaves health below the maximum.',
 'colors.maxHealth.description':'Particle color when healing reaches maximum health.'}

def update(language,path):
 data=json.loads(path.read_text(encoding='utf8'))
 put_module(data,'liquidbounce.module.damageParticles','DamageParticles',CN if language=='zh_cn'else TW,EN)
 path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
def main():
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,Path(a.directory)/f'display_{language}.json')
if __name__=='__main__':main()
