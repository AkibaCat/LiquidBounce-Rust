#!/usr/bin/env python3
"""ReverseStep simplified/traditional display vocabulary."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'MaximumFallDistance':('最大落差','最大落差'),'Instant':('瞬间','瞬間'),'Strict':('固定下降','固定下降'),'Accelerator':('下降加速','下降加速'),'SimulateFalling':('模拟下落数据包','模擬下落封包'),'Factor':('倍率','倍率')}
TEXTS={
 'reverseStep':('Allows you to step down blocks faster.','更快地从方块边缘落下。','更快地從方塊邊緣落下。'),
 'reverseStep.mode':('Instant predicts the landing, Strict sets downward velocity, and Accelerator multiplies existing downward velocity.','瞬间模式预测并移动至落点；固定下降模式设置下降速度；下降加速模式按倍率增加已有下降速度。','瞬間模式預測並移動至落點；固定下降模式設定下降速度；下降加速模式按倍率增加已有下降速度。'),
 'reverseStep.maximumFallDistance':('Maximum fall distance. ReverseStep requires block collision at that distance and avoids deliberate jumps. Default 1 block.','允许的最大落差；该距离处必须有方块碰撞，且不会干预主动跳跃，默认 1 格。','允許的最大落差；該距離處必須有方塊碰撞，且不會干預主動跳躍，預設 1 格。'),
 'reverseStep.mode.instant.ticks':('Maximum prediction ticks to find a landing. Default 20.','寻找落点的最大预测刻数，默认 20 刻。','尋找落點的最大預測刻數，預設 20 刻。'),
 'reverseStep.mode.instant.simulateFalling':('Send the successful predicted falling path as movement packets before moving to the landing.','移动至落点前，按成功预测的下落轨迹发送移动数据包。','移動至落點前，按成功預測的下落軌跡傳送移動封包。'),
 'reverseStep.mode.strict.motion':('Downward movement speed. Default 1.','固定的下降速度，默认 1。','固定的下降速度，預設 1。'),
 'reverseStep.mode.accelerator.factor':('Multiply downward velocity by this factor and clear horizontal velocity. Default 1.','将下降速度乘以该倍率，并清除水平速度，默认 1。','將下降速度乘以該倍率，並清除水平速度，預設 1。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  data['moduleNames']['ReverseStep']=('快速下阶','快速下階')[index-1]
  data['labels'].update({k:v[index-1] for k,v in LABELS.items()})
  for name,texts in TEXTS.items():
   key=f'liquidbounce.module.{name}.description';data['descriptions'][key]=texts[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=texts[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
