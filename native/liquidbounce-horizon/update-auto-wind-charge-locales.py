#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={'AutoWindCharge':('自动风弹跳跃','自動風彈跳躍'),'HorizontalBoost':('水平推进','水平推進'),'CombatPauseTime':('暂停战斗时间','暫停戰鬥時間'),'SlotResetDelay':('恢复快捷栏延迟','恢復快捷列延遲')}
TEXTS={
 'description':('Use wind charges to boost yourself while holding jump.','按住跳跃时自动使用风弹推进自己。','按住跳躍時自動使用風彈推進自己。'),
 'horizontalBoost.description':('Hold the configured key while jumping to aim opposite your movement and gain horizontal speed.','跳跃时按住指定按键，向移动方向的反方向投掷以获得水平速度。','跳躍時按住指定按鍵，朝移動方向的反方向投擲以獲得水平速度。'),
 'horizontalBoost.enabled.description':('Enable the optional horizontal wind-charge boost.','启用可按键触发的风弹水平推进。','啟用可按鍵觸發的風彈水平推進。'),
 'horizontalBoost.pitch.description':('Random pitch used for horizontal boosts; it is kept until a charge is thrown.','水平推进使用的随机俯仰角；选定后保持至投出风弹。','水平推進使用的隨機俯仰角；選定後保持至投出風彈。'),
 'horizontalBoost.key.description':('Hold this key together with jump to use horizontal boosting.','同时按住此键与跳跃键以触发水平推进。','同時按住此鍵與跳躍鍵以觸發水平推進。'),
 'rotate.description':('Turn before landing and wait on the ground for up to ten ticks before allowing a jump without a charge.','落地前提前转向；落地后最多等待十刻瞄准，超时则允许直接跳跃。','落地前提前轉向；落地後最多等待十刻瞄準，逾時則允許直接跳躍。'),
 'rotate.enabled.description':('Aim before throwing and wait until the server view direction is within one degree.','投掷前转向，并等待服务器视线方向误差不超过一度。','投擲前轉向，並等待伺服器視線方向誤差不超過一度。'),
 'combatPauseTime.description':('Pause automatic combat for at least this many ticks whenever requesting aim.','每次请求瞄准时，至少暂停自动战斗这么多游戏刻。','每次請求瞄準時，至少暫停自動戰鬥這麼多遊戲刻。'),
 'slotResetDelay.description':('Random delay before releasing the silent hotbar selection used to throw the charge.','投掷风弹时静默选择快捷栏后，随机等待这么多刻再恢复。','投擲風彈時靜默選擇快捷列後，隨機等待這麼多刻再恢復。')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoWindCharge'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.autoWindCharge','AutoWindCharge',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    if key.startswith(old):data[field][key.replace(old,'liquidbounce.module.autoWindCharge.rotate.rotations')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
