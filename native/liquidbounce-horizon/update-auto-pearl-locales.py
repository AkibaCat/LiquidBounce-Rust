#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={'AutoPearl':('自动跟随珍珠','自動跟隨珍珠'),'PearlFollower':('跟随珍珠','跟隨珍珠'),'PearlTarget':('目标珍珠','目標珍珠'),'Limits':('限制','限制'),'MinDistance':('最小距离','最小距離'),'DestinationDistance':('落点允许偏差','落點允許偏差'),'CombatPauseTime':('暂停战斗时间','暫停戰鬥時間'),'SlotResetDelay':('恢复快捷栏延迟','恢復快捷列延遲')}
TEXTS={
 'description':('Follow an incoming ender pearl by predicting its impact and throwing a pearl toward it.','预测飞来末影珍珠的落点，并投掷自己的珍珠跟随。','預測飛來終界珍珠的落點，並投擲自己的珍珠跟隨。'),
 'mode.description':('Trigger follows attackable owners or ownerless pearls; Target follows the actual KillAura target.','触发模式跟随可攻击实体或无主珍珠；目标模式跟随 KillAura 当前真实目标。','觸發模式跟隨可攻擊實體或無主珍珠；目標模式跟隨 KillAura 目前真實目標。'),
 'mode.value.trigger.description':('Follow pearls from attackable entities and pearls with no resolved owner.','跟随可攻击实体投出的珍珠，以及未解析到主人的珍珠。','跟隨可攻擊實體投出的珍珠，以及未解析到主人的珍珠。'),
 'mode.value.target.description':('Follow only pearls owned by the current KillAura target.','仅跟随 KillAura 当前目标投出的珍珠。','僅跟隨 KillAura 目前目標投出的珍珠。'),
 'rotate.description':('Turn toward the solved throw rotation before throwing.','投掷前转向计算出的投掷角度。','投擲前轉向計算出的投擲角度。'),
 'rotate.enabled.description':('Wait up to twenty ticks until the server view is within one degree; abort on timeout.','最多等待二十刻，直到服务器视线误差不超过一度；超时则放弃。','最多等待二十刻，直到伺服器視線誤差不超過一度；逾時則放棄。'),
 'limits.description':('Restrict pearl angle, activation distance and simulated destination error.','限制珍珠视角、触发距离及模拟落点偏差。','限制珍珠視角、觸發距離及模擬落點偏差。'),
 'limits.enabled.description':('Apply the angle and distance restrictions; both real trajectories still require an impact.','启用视角与距离限制；两条真实轨迹仍必须碰撞到目标。','啟用視角與距離限制；兩條真實軌跡仍必須碰撞到目標。'),
 'limits.angle.description':('Maximum angle from the crosshair to the incoming pearl bounding box.','准星方向与飞来珍珠碰撞箱之间的最大夹角。','準星方向與飛來珍珠碰撞箱之間的最大夾角。'),
 'limits.minDistance.description':('Ignore an incoming pearl whose predicted impact is strictly nearer than this distance from your feet.','忽略预测落点与玩家脚部距离小于此值的珍珠。','忽略預測落點與玩家腳部距離小於此值的珍珠。'),
 'limits.destinationDistance.description':('Your simulated impact must be strictly closer than this distance to the incoming pearl impact.','自己模拟的落点与对方落点之间的距离必须严格小于此值。','自己模擬的落點與對方落點之間的距離必須嚴格小於此值。'),
 'combatPauseTime.description':('Pause automatic combat for at least this many ticks while a throw waits in the queue.','投掷仍在队列中时，至少暂停自动战斗这么多刻。','投擲仍在佇列中時，至少暫停自動戰鬥這麼多刻。'),
 'slotResetDelay.description':('Random delay before releasing the silent hotbar selection used for the pearl.','投掷珍珠后，静默快捷栏选择保持的随机刻数。','投擲珍珠後，靜默快捷列選擇保持的隨機刻數。')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoPearl'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.autoPearl','AutoPearl',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    if key.startswith(old):data[field][key.replace(old,'liquidbounce.module.autoPearl.rotate.rotations')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
