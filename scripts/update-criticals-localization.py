#!/usr/bin/env python3
"""Criticals original settings, in Simplified and Traditional Chinese."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'WhenSprinting':('冲刺时','衝刺時'),'StopSprinting':('停止冲刺','停止衝刺'),'Visuals':('视觉效果','視覺效果'),'Fake':('模拟暴击效果','模擬暴擊效果'),
 'OptimizeForCooldown':('根据冷却优化','依冷卻最佳化'),'CheckKillaura':('检查杀戮光环','檢查殺戮光環'),'CheckAutoClicker':('检查自动连点','檢查自動連點'),'CheckTriggerBot':('检查准星自动攻击','檢查準星自動攻擊'),'CanBeSeen':('要求视线可见','要求視線可見'),
 'OnNetwork':('发包时停止','發包時停止'),'OnAttack':('攻击时停止','攻擊時停止'),'NoGround':('无着地标记','無著地標記'),'Rewound':('回溯','回溯'),
 'OnGroundOnly':('仅着地状态','僅著地狀態'),'PositionAndOnGround':('位置与着地状态','位置與著地狀態'),'LookAndOnGround':('朝向与着地状态','朝向與著地狀態'),
}
TEXTS={
 '':('Automatically creates or times critical hits with the selected mode.','使用所选模式自动制造暴击或调整攻击时机。','使用所選模式自動製造暴擊或調整攻擊時機。'),
 'mode.packet.mode':('Choose the movement offset sequence sent before an attack.','选择攻击前发送的移动偏移序列。','選擇攻擊前傳送的移動偏移序列。'),
 'mode.packet.packetType':('Choose which position and rotation fields the critical movement packets carry.','选择暴击移动包携带的位置和朝向字段。','選擇暴擊移動封包攜帶的位置和朝向欄位。'),
 'mode.jump.height':('Vertical motion for automatic normal jumps.','自动执行普通跳跃时的竖直速度。','自動執行普通跳躍時的垂直速度。'),
 'mode.jump.range':('Look for attackable enemies within this distance before automatically jumping.','在此距离内寻找可攻击敌人，再自动跳跃。','在此距離內尋找可攻擊敵人，再自動跳躍。'),
 'mode.jump.optimizeForCooldown':('Predict landing and attack cooldown before starting a jump.','开始跳跃前预测落地时间与攻击冷却。','開始跳躍前預測著地時間與攻擊冷卻。'),
 'mode.jump.checkKillaura':('Require KillAura to be running when this module check is selected.','选中此检查时，要求杀戮光环正在运行。','選取此檢查時，要求殺戮光環正在執行。'),
 'mode.jump.checkAutoClicker':('Allow automatic jumps while AutoClicker is running.','自动连点运行时允许自动跳跃。','自動連點執行時允許自動跳躍。'),
 'mode.jump.checkTriggerBot':('Allow automatic jumps while TriggerBot is running.','准星自动攻击运行时允许自动跳跃。','準星自動攻擊執行時允許自動跳躍。'),
 'mode.jump.canBeSeen':('Require a clear line of sight to an enemy before automatically jumping.','自动跳跃前要求敌人处于无遮挡视线内。','自動跳躍前要求敵人處於無遮擋視線內。'),
 'mode.blink.delay':('Maximum time to queue packets before releasing them and choosing the next delay.','释放排队数据包并重新抽取延迟前，允许的最长排队时间。','釋放排隊封包並重新抽取延遲前，允許的最長排隊時間。'),
 'mode.blink.range':('Queue packets near an attackable enemy while a critical hit is unavailable.','附近存在可攻击敌人且暂时无法暴击时，对数据包排队。','附近存在可攻擊敵人且暫時無法暴擊時，將封包排隊。'),
 'mode.timer.speed':('Temporary tick speed while a critical hit is available near an enemy.','附近存在敌人且可以暴击时，临时采用的游戏刻速度。','附近存在敵人且可以暴擊時，暫時採用的遊戲刻速度。'),
 'mode.timer.range':('Distance within which an attackable enemy can activate the critical timer.','可触发暴击计时器的敌人距离。','可觸發暴擊計時器的敵人距離。'),
 'whenSprinting.enabled':('Adjust sprinting when falling and ready for a critical hit near an enemy.','附近存在敌人且下落时已可暴击，调整冲刺状态。','附近存在敵人且下落時已可暴擊，調整衝刺狀態。'),
 'whenSprinting.stopSprinting':('Choose which input, movement, network or attack event stops sprinting.','选择在哪个输入、移动、网络或攻击事件中停止冲刺。','選擇在哪個輸入、移動、網路或攻擊事件中停止衝刺。'),
 'whenSprinting.range':('Enemy distance required for sprint adjustment.','触发冲刺调整所需的敌人距离。','觸發衝刺調整所需的敵人距離。'),
 'visuals.enabled':('Show critical and magic hit particles on attacked living entities.','在被攻击的生物上显示暴击与魔法命中粒子。','在被攻擊的生物上顯示暴擊與魔法命中粒子。'),
 'visuals.fake':('Show the configured particles even when the hit is not critical.','普通攻击也显示所配置的粒子效果。','普通攻擊也顯示所設定的粒子效果。'),
 'visuals.critical':('Number of critical particle emitters for each visual effect.','每次视觉效果创建的暴击粒子发射器数量。','每次視覺效果建立的暴擊粒子發射器數量。'),
 'visuals.magic':('Number of magic hit particle emitters for each visual effect.','每次视觉效果创建的魔法命中粒子发射器数量。','每次視覺效果建立的魔法命中粒子發射器數量。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['Criticals']=('自动暴击','自動暴擊')[index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for suffix,v in TEXTS.items():
   key='liquidbounce.module.criticals'+('.'+suffix if suffix else '')+'.description';data['descriptions'][key]=v[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=v[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'Criticals descriptions')
if __name__=='__main__':main()
