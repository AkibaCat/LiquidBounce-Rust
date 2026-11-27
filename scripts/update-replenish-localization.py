#!/usr/bin/env python3
"""Complete Replenish Chinese labels and source-key descriptions."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'ItemThreshold':('补充阈值','補充門檻'),'ReplenishEmpty':('补充空槽','補充空欄位'),
 'Features':('补充方式','補充方式'),'CleanUp':('优先清理小堆叠','優先清理小堆疊'),
 'UsePickupAll':('双击合并','連按兩下合併'),'UseSwap':('直接交换','直接交換'),
 'InsideOf':('允许的界面','允許的介面'),'Chests':('容器界面','容器介面'),'Inventories':('玩家物品栏','玩家物品欄'),
}
TEXTS={
 'replenish':('Refill the offhand and hotbar from the main inventory.','从主物品栏自动补充副手和快捷栏物品。','從主物品欄自動補充副手和快捷列物品。'),
 'replenish.itemThreshold':('Refill stacks at or below this count when their maximum stack size exceeds the threshold. Default 5.','物品数量不超过此值时补充；最大堆叠数不超过阈值的非空物品不补充。默认 5。','物品數量不超過此值時補充；最大堆疊數不超過門檻的非空物品不補充。預設 5。'),
 'replenish.delay':('Real-time milliseconds between checks and after each refill, resumed on the game tick. Default 40.','检查间隔和每个槽位补充后的实际等待毫秒数，在游戏刻恢复，默认 40。','檢查間隔和每個欄位補充後的實際等待毫秒數，在遊戲刻恢復，預設 40。'),
 'replenish.replenishEmpty':('Refill empty slots using the last tracked item and its components. Opening a container clears this memory.','记住物品及其组件并补充已用空的槽位；开启容器界面时清除此记忆。','記住物品及其元件並補充已用空的欄位；開啟容器介面時清除此記憶。'),
 'replenish.features':('CleanUp consumes smaller inventory stacks first. UseSwap prefers a larger replacement; UsePickupAll merges by double-click.','清理模式优先使用小堆叠；直接交换优先换入更大的一组；双击合并收集同类物品。默认只启用清理模式。','清理模式優先使用小堆疊；直接交換優先換入較大的一組；連按兩下合併收集同類物品。預設只啟用清理模式。'),
 'replenish.insideOf':('Allow this module to run while container or player-inventory screens are open. Actual actions still obey inventory-manager restrictions.','允许模块在选定的容器或玩家物品栏界面中运行；实际点击仍遵守库存管理器限制。默认均关闭。','允許模組在選定的容器或玩家物品欄介面中執行；實際點擊仍遵守庫存管理器限制。預設皆關閉。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  data['moduleNames']['Replenish']=('快捷栏补充','快捷列補充')[index-1]
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   if field not in data:continue
   for key,text in list(data[field].items()):
    if key.startswith('liquidbounce.module.elytraSwap.constraints'):
     data[field][key.replace('.elytraSwap.','.replenish.')]=text
  for suffix,text in TEXTS.items():
   key=f'liquidbounce.module.{suffix}.description';data['descriptions'][key]=text[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
