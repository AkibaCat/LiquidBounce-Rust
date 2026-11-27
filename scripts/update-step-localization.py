#!/usr/bin/env python3
"""Chinese Step display vocabulary; machine configuration identifiers stay stable."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'Legit':('原版自动跳跃','原版自動跳躍'),'SimulateJumpOrder':('模拟跳跃阶段','模擬跳躍階段'),'Trim':('限制模拟高度','限制模擬高度'),'Wait':('间隔','間隔'),'PacketType':('数据包类型','封包類型'),'PositionAndOnGround':('位置与着地状态','位置與著地狀態'),'Full':('完整移动数据包','完整移動封包'),'BaseTimer':('起步时钟倍率','起步時鐘倍率'),'RecoveryTimer':('恢复时钟倍率','恢復時鐘倍率'),'AlternateBypass':('替代方式','替代方式'),'Spoof':('伪装着地','偽裝著地')}
TEXTS={
 'step':('Allows you to step up blocks.','自动走上更高的方块。','自動走上更高的方塊。'),
 'step.mode':('Choose Instant, vanilla auto jump, Vulcan, BlocksMC or Hypixel stepping.','选择瞬间、原版自动跳跃、Vulcan、BlocksMC 或 Hypixel 上阶模式。','選擇瞬間、原版自動跳躍、Vulcan、BlocksMC 或 Hypixel 上階模式。'),
 'step.mode.instant.height':('Maximum step height. Default 1 block.','最大上阶高度，默认 1 格。','最大上階高度，預設 1 格。'),
 'step.mode.instant.trim':('Cap simulated jump packet heights at the actual step destination.','将模拟跳跃数据包的高度限制在本次台阶的实际终点。','將模擬跳躍封包的高度限制在本次台階的實際終點。'),
 'step.mode.instant.simulateJumpOrder':('Inclusive jump trajectory indices to send. 0..0 sends no packets; default 0..2.','发送的跳跃轨迹阶段（包含两端）；0..0 不发送，默认 0..2。','傳送的跳躍軌跡階段（包含兩端）；0..0 不傳送，預設 0..2。'),
 'step.mode.instant.wait':('Random cooldown between height overrides, in game ticks. Default 0..0.','两次调整上阶高度之间的随机等待刻数，默认 0..0。','兩次調整上階高度之間的隨機等待刻數，預設 0..0。'),
 'step.mode.instant.packetType':('Use full movement or position and ground packets for the simulated jump.','模拟跳跃使用完整移动数据包，或仅位置与着地状态。','模擬跳躍使用完整移動封包，或僅位置與著地狀態。'),
 'step.mode.blocksMC.baseTimer':('Timer speed requested during the first three stepping ticks. Default 3.','上阶开始阶段请求的时钟倍率（持续 3 刻），默认 3。','上階開始階段請求的時鐘倍率（持續 3 刻），預設 3。'),
 'step.mode.blocksMC.recoveryTimer':('Timer speed requested for two ticks after stepping. Default 0.6.','上阶后恢复阶段请求的时钟倍率（持续 2 刻），默认 0.6。','上階後恢復階段請求的時鐘倍率（持續 2 刻），預設 0.6。'),
 'step.mode.hypixel-2025-03-11.alternateBypass':('After a 1.5 block step, set the body on ground instead of repeating upward velocity.','跨越 1.5 格台阶后，使用着地状态作为最后阶段。','跨越 1.5 格台階後，使用著地狀態作為最後階段。'),
 'step.mode.hypixel-2025-03-11.spoof':('Report on ground on the eighth airborne movement tick.','在第 8 个空中移动刻发送着地状态。','在第 8 個空中移動刻傳送著地狀態。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['Step']=('自动上阶','自動上階')[index-1]
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for name,texts in TEXTS.items():
   key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=texts[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=texts[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
