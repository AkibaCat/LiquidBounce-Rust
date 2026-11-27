#!/usr/bin/env python3
"""Add AutoBow's complete display strings without changing stored identifiers."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={
 'AutoBow':('自动弓箭','自動弓箭'),'AutoShoot':('自动射击','自動射擊'),'BowAimbot':('弹道瞄准','彈道瞄準'),'FastCharge':('快速蓄力','快速蓄力'),
 'Charged':('蓄力时间','蓄力時間'),'ChargedRandom':('随机蓄力偏移','隨機蓄力偏移'),'DelayBetweenShots':('射击间隔','射擊間隔'),'AimThreshold':('瞄准角度阈值','瞄準角度門檻'),
 'RequiresHypotheticalHit':('要求预测命中','要求預測命中'),'UsePrechargedCrossbow':('使用已装填弩','使用已裝填弩'),'ThroughWalls':('允许穿墙解算','允許穿牆解算'),
 'Target':('目标选择','目標選擇'),'Priority':('优先级','優先順序'),'Type':('类型','類型'),'Health':('生命值','生命值'),'Distance':('距离','距離'),'Direction':('方向','方向'),'HurtTime':('受伤时间','受傷時間'),'Age':('存在时间','存在時間'),
 'NotInTheAir':('空中停用','空中停用'),'NotDuringMove':('移动时停用','移動時停用'),'NotDuringRegeneration':('生命恢复时停用','生命恢復時停用'),
 'PacketType':('封包类型','封包類型'),'OnGroundOnly':('仅落地状态','僅落地狀態'),'PositionAndOnGround':('位置与落地状态','位置與落地狀態'),'LookAndOnGround':('朝向与落地状态','朝向與落地狀態'),'Full':('完整移动封包','完整移動封包'),
}
TEXTS={}
def text(path,en,cn,tw):TEXTS[path+'.description'if path else'description']=(en,cn,tw)
text('','Aim projectile weapons with motion prediction, release shots automatically, and optionally accelerate bow charging.','预测目标运动并瞄准弹射武器，自动松弓射击，并可加速弓的蓄力。','預測目標運動並瞄準投射武器，自動鬆弓射擊，並可加速弓的蓄力。')
text('autoShoot','Configure automatic release of bows, crossbows and tridents.','配置弓、弩与三叉戟的自动射击。','設定弓、弩與三叉戟的自動射擊。')
text('autoShoot.enabled','Automatically shoot when charge, delay and aiming conditions are satisfied.','蓄力、射击间隔及瞄准条件满足时自动射击。','蓄力、射擊間隔及瞄準條件滿足時自動射擊。')
text('autoShoot.charged','Ticks to draw a bow before releasing, with the cached random offset added.','松弓前的基础蓄力游戏刻数；会加上当前缓存的随机偏移。','鬆弓前的基本蓄力遊戲刻數；會加上目前快取的隨機偏移。')
text('autoShoot.chargedRandom','Gaussian random offset added to bow charge time, refreshed after each automatic bow shot.','以高斯分布生成弓蓄力时间偏移，每次自动射箭后重新生成。','以高斯分布產生弓蓄力時間偏移，每次自動射箭後重新產生。')
text('autoShoot.delayBetweenShots','Minimum elapsed seconds since the previous shot or since a bow stopped being used.','自上次射击或停止使用弓以来必须经过的最短秒数。','自上次射擊或停止使用弓以來必須經過的最短秒數。')
text('autoShoot.aimThreshold','Maximum angle between server aim and the active managed rotation when the aimbot is enabled.','启用弹道瞄准时，服务器朝向与当前管理旋转之间允许的最大角度。','啟用彈道瞄準時，伺服器朝向與目前管理旋轉之間允許的最大角度。')
text('autoShoot.requiresHypotheticalHit','Require the forty-tick projectile simulation to hit a valid target, instead of checking managed aim.','使用四十刻弹射物模拟确认可命中有效目标，替代管理旋转的角度检查。','使用四十刻投射物模擬確認可命中有效目標，取代管理旋轉的角度檢查。')
text('autoShoot.usePrechargedCrossbow','Allow shooting an already loaded crossbow from either hand without holding use.','即使没有按住使用键，也允许发射任意手中已经装填的弩。','即使沒有按住使用鍵，也允許發射任意手中已經裝填的弩。')
text('bowAimbot','Calculate projectile aim using target motion, gravity, drag and visible points.','根据目标运动、重力、阻力与可见瞄准点计算弹道。','根據目標運動、重力、阻力與可見瞄準點計算彈道。')
text('bowAimbot.enabled','Aim while using a bow, crossbow or trident, or while holding a loaded crossbow.','使用弓、弩、三叉戟或手持已装填弩时自动瞄准。','使用弓、弩、三叉戟或手持已裝填弩時自動瞄準。')
text('bowAimbot.throughWalls','Skip full-trajectory collision verification; the solver still needs a visible approach point near the target.','跳过整条弹道的碰撞验证；解算器仍要求目标附近有可见的接近瞄准点。','略過整條彈道的碰撞驗證；解算器仍要求目標附近有可見的接近瞄準點。')
text('bowAimbot.target','Configure eligible targets and their ordered sorting criteria.','配置可选目标及其排序条件的先后顺序。','設定可選目標及其排序條件的先後順序。')
text('bowAimbot.target.fov','Maximum angular distance from the current view to the nearest part of a target hitbox.','当前视线与目标碰撞箱最近可瞄准部分之间允许的最大夹角。','目前視線與目標碰撞箱最近可瞄準部分之間允許的最大夾角。')
text('bowAimbot.target.hurtTime','Reject targets whose remaining damage immunity exceeds this many ticks.','目标剩余受伤免疫游戏刻数高于此值时不选择。','目標剩餘受傷免疫遊戲刻數高於此值時不選擇。')
text('bowAimbot.target.priority','Apply the selected criteria in order; later criteria resolve ties, and at least one is required.','按选中顺序依次排序；后续条件用于打破前项相同的情况，至少选择一项。','按選取順序依次排序；後續條件用於區分前項相同的情況，至少選擇一項。')
for key,en,cn,tw in [
 ('type','Prefer players, then hostile mobs, then other entities.','优先玩家，其次敌对生物，最后其他实体。','優先玩家，其次敵對生物，最後其他實體。'),
 ('health','Prefer lower health, using a recognized below-name health objective when available.','优先较低生命值；识别到名称下方的生命值计分板时使用其数值。','優先較低生命值；辨識到名稱下方的生命值計分板時使用其數值。'),
 ('distance','Prefer the target hitbox closest to the player eye.','优先碰撞箱离玩家眼睛最近的目标。','優先碰撞箱離玩家眼睛最近的目標。'),
 ('direction','Prefer the hitbox requiring the smallest change from the current view.','优先与当前视线夹角最小的目标碰撞箱。','優先與目前視線夾角最小的目標碰撞箱。'),
 ('hurtTime','Prefer targets with less remaining damage immunity.','优先剩余受伤免疫时间更短的目标。','優先剩餘受傷免疫時間更短的目標。'),
 ('age','Prefer entities that have existed for more ticks.','优先已存在游戏刻数更多的实体。','優先已存在遊戲刻數更多的實體。')]:text('bowAimbot.target.priority.'+key,en,cn,tw)
text('fastCharge','Accelerate bow use by sending movement packets and updating the active item repeatedly.','反复发送移动封包并更新当前使用物品，以加快弓的蓄力。','反覆傳送移動封包並更新目前使用物品，以加快弓的蓄力。')
text('fastCharge.enabled','Enable accelerated charging while using a bow.','使用弓时启用快速蓄力。','使用弓時啟用快速蓄力。')
text('fastCharge.speed','Extra movement packets and item-use updates attempted each game tick.','每个游戏刻额外尝试发送的移动封包数与物品使用更新次数。','每個遊戲刻額外嘗試傳送的移動封包數與物品使用更新次數。')
text('fastCharge.notInTheAir','Only accelerate charging while the player is on the ground.','仅在玩家着地时加快蓄力。','僅在玩家著地時加快蓄力。')
text('fastCharge.notDuringMove','Only accelerate charging while the player has no movement input.','仅在玩家没有移动输入时加快蓄力。','僅在玩家沒有移動輸入時加快蓄力。')
text('fastCharge.notDuringRegeneration','Suspend accelerated charging while the Regeneration effect is active.','生命恢复效果生效期间暂停快速蓄力。','生命恢復效果生效期間暫停快速蓄力。')
text('fastCharge.packetType','Select the movement-packet fields sent for each accelerated charge update.','选择每次快速蓄力更新所发送移动封包包含的字段。','選擇每次快速蓄力更新所傳送移動封包包含的欄位。')

def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoBow'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.autoBow','AutoBow',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
   for key,value in list(data.get(field,{}).items()):
    if key.startswith(old):data[field][key.replace(old,'liquidbounce.module.autoBow.bowAimbot.rotations')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
