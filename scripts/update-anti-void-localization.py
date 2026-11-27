#!/usr/bin/env python3
"""AntiVoid Simplified/Traditional Chinese labels, descriptions and English fallback."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'AntiVoid':('防坠虚空','防墜虛空'),'GhostBlock':('虚拟方块','虛擬方塊'),
 'Flag':('抬升触发','抬升觸發'),'Blink':('延迟回退','延遲回退'),'UseItem':('使用物品','使用物品'),
 'VoidLevel':('虚空高度','虛空高度'),'FallDistance':('下落距离','下落距離'),
 'Height':('抬升高度','抬升高度'),'Silent':('静默抬升','靜默抬升'),
 'YMotionThreshold':('垂直速度阈值','垂直速度門檻'),'SlotResetDelay':('槽位恢复延迟','欄位還原延遲'),
 'Items':('物品列表','物品清單'),
}
TEXTS={
 '':('Predict a void fall and rescue with GhostBlock, Flag, Blink or UseItem. Disabled by default.','预测坠入虚空并通过虚拟方块、抬升、延迟回退或使用物品进行保护，默认关闭。','預測墜入虛空並透過虛擬方塊、抬升、延遲回退或使用物品進行保護，預設關閉。'),
 'Mode':('Select the rescue mode. Default GhostBlock. All modes predict the first falling snapshot among ticks 0–9.','选择保护模式，默认虚拟方块。所有模式检查第 0–9 刻中的首个下落快照。','選擇保護模式，預設虛擬方塊。所有模式檢查第 0–9 刻中的首個下落快照。'),
 'VoidLevel':('Bottom Y coordinate of each collision column. Default 0; range -256–0.','碰撞检测柱的底部 Y 坐标，默认 0，范围 -256–0。','碰撞偵測柱的底部 Y 座標，預設 0，範圍 -256–0。'),
 'Mode.GhostBlock':('Give empty collision shapes below the last safe position a full-block shape while a void fall is predicted.','预测将坠入虚空时，将上次安全位置下方的空碰撞形状变为完整方块。','預測將墜入虛空時，將上次安全位置下方的空碰撞形狀變為完整方塊。'),
 'Mode.Flag':('Raise the player or the next movement event after the configured fall distance.','达到所设下落距离后，抬升玩家或下一次移动事件的高度。','達到所設下落距離後，抬升玩家或下一次移動事件的高度。'),
 'Mode.Flag.FallDistance':('Minimum fall distance before Flag rescue. Default 0.5; range 0–6.','触发抬升保护的最小下落距离，默认 0.5，范围 0–6。','觸發抬升保護的最小下落距離，預設 0.5，範圍 0–6。'),
 'Mode.Flag.Height':('Y offset added during rescue. Default 0.42; range 0.01–10.','保护时增加的 Y 偏移，默认 0.42，范围 0.01–10。','保護時增加的 Y 偏移，預設 0.42，範圍 0.01–10。'),
 'Mode.Flag.Silent':('Apply Height to the next PRE movement event instead of moving the local body. Default false.','将抬升高度应用于下一次 PRE 移动事件，默认关闭。','將抬升高度套用於下一次 PRE 移動事件，預設關閉。'),
 'Mode.Blink':('Queue outgoing packets while a void fall is predicted; cancel the shared Blink queue to return to its first position. Scaffold exempts this mode.','预测坠入虚空时延迟发出的数据包；取消共享延迟队列以回到首个位置。搭路运行时豁免。','預測墜入虛空時延遲送出的封包；取消共用延遲佇列以回到首個位置。搭路執行時豁免。'),
 'Mode.UseItem':('Use a configured item when falling fast and no ground is predicted in ticks 1–40.','下落速度足够快，且预测第 1–40 刻均无法落地时使用所选物品。','下落速度足夠快，且預測第 1–40 刻均無法落地時使用所選物品。'),
 'Mode.UseItem.YMotionThreshold':('Require vertical velocity below this threshold. Default -0.5; range -1.0–-0.1.','要求垂直速度低于此阈值，默认 -0.5，范围 -1.0–-0.1。','要求垂直速度低於此門檻，預設 -0.5，範圍 -1.0–-0.1。'),
 'Mode.UseItem.SlotResetDelay':('Random silent hotbar reset delay. Default 4–6; endpoints 0–40 ticks.','静默选择快捷栏后的随机恢复延迟，默认 4–6，两端范围 0–40 刻。','靜默選擇快捷欄後的隨機還原延遲，預設 4–6，兩端範圍 0–40 刻。'),
 'Mode.UseItem.Items':('Accepted item registry entries. Default magma cream. Prefer the selected hand, then offhand, then the nearest hotbar slot.','允许使用的物品，默认岩浆膏。优先当前选中槽位，其次副手，再按快捷栏距离选择。','允許使用的物品，預設岩漿球。優先目前選取欄位，其次副手，再依快捷欄距離選擇。'),
}
def description_key(path):
    suffix='.'.join(part[:1].lower()+part[1:]for part in path.split('.'))
    return 'liquidbounce.module.antiVoid'+('.'+suffix if suffix else '')+'.description'
def update(data,index):
    data['moduleNames']['AntiVoid']=LABELS['AntiVoid'][index-1]
    data['labels'].update({key:text[index-1]for key,text in LABELS.items()})
    for path,text in TEXTS.items():
        key=description_key(path);data['descriptions'][key]=text[index]
        if index==1:data['englishDescriptions'][key]=text[0]
    return data
def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json'
        data=update(json.loads(path.read_text(encoding='utf8')),index)
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        print(locale,len(TEXTS),'AntiVoid descriptions')
if __name__=='__main__':main()
