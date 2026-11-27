#!/usr/bin/env python3
"""Complete AutoDeposit Chinese labels and source-key descriptions."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'PunchToDeposit':('左键存入','左鍵存入'),'ValidStorageBlocks':('可存储方块','可儲存方塊'),'RotateToTarget':('转向目标','轉向目標'),'PauseOn':('暂停条件','暫停條件'),'WallRange':('穿墙范围','穿牆範圍'),
 'DoNotHide':('正常挥手','正常揮手'),'HideForBoth':('隐藏全部挥手','隱藏所有揮手'),'HideForClient':('仅隐藏本地挥手','僅隱藏本機揮手'),'HideForServer':('仅隐藏服务器挥手','僅隱藏伺服器揮手'),'UsingItem':('使用物品时','使用物品時'),
}
TEXTS={
 'autoDeposit':('Deposit matching inventory items into accepted containers. PunchToDeposit sends server-specific storage clicks.','自动将符合筛选条件的物品存入允许的容器；左键存入功能适用于支持该行为的服务器。','自動將符合篩選條件的物品存入允許的容器；左鍵存入功能適用於支援此行為的伺服器。'),
 'autoDeposit.filter':('Select an item whitelist or blacklist.','选择物品白名单或黑名单。','選擇物品白名單或黑名單。'),
 'autoDeposit.items':('Items matched by the whitelist or blacklist.','自动存入的物品筛选列表。','自動存入的物品篩選清單。'),
 'autoDeposit.autoClose':('Close only a container into which this module deposited an item.','只在此模块向当前容器存入过物品后自动关闭；容器已满时不会反复点击。','只在此模組向目前容器存入過物品後自動關閉；容器已滿時不會反覆點擊。'),
 'autoDeposit.punchToDeposit.enabled':('Punch supported storage blocks to deposit a matching hotbar stack.','向支持该功能的存储方块发送完整左键点击，将快捷栏中符合条件的一组物品存入。','向支援此功能的儲存方塊傳送完整左鍵點擊，將快捷列中符合條件的一組物品存入。'),
 'autoDeposit.punchToDeposit.range':('Target selection range. Default 3; editor range 1..6.','自动选择目标的范围，默认 3 格，界面范围 1..6 格。','自動選擇目標的範圍，預設 3 格，介面範圍 1..6 格。'),
 'autoDeposit.punchToDeposit.wallRange':('Target selection range through walls, limited by Range when changed. Default 0.','穿墙选择目标的范围，修改时不超过交互范围，默认 0。','穿牆選擇目標的範圍，修改時不超過互動範圍，預設 0。'),
 'autoDeposit.punchToDeposit.delay':('Ticks to wait after a storage click. Default 5; editor range 1..80.','每次左键存入后等待的游戏刻数，默认 5，界面范围 1..80。','每次左鍵存入後等待的遊戲刻數，預設 5，介面範圍 1..80。'),
 'autoDeposit.punchToDeposit.swingMode':('Select local and server attack-swing visibility.','选择本地与服务器端的攻击挥手显示方式。','選擇本機與伺服器端的攻擊揮手顯示方式。'),
 'autoDeposit.punchToDeposit.pauseOn':('Pause storage clicks during the selected conditions.','满足选定条件时暂停左键存入。','符合選定條件時暫停左鍵存入。'),
 'autoDeposit.punchToDeposit.validStorageBlocks':('Storage block types to target. Defaults: chest and ender chest.','可作为目标的存储方块类型，默认普通箱子和末影箱。','可作為目標的儲存方塊類型，預設儲物箱和終界箱。'),
 'autoDeposit.punchToDeposit.rotateToTarget':('Find an accessible storage block and request rotation through the shared rotation manager.','自动寻找可触及的存储方块，并通过转头管理器转向目标；关闭时使用准星目标。','自動尋找可觸及的儲存方塊，並透過轉向管理器轉向目標；關閉時使用準星目標。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  data['moduleNames']['AutoDeposit']=('自动存入','自動存入')[index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   if field not in data:continue
   for key,text in list(data[field].items()):
    if key.startswith('liquidbounce.module.chestCleaner.') and any(p in key for p in ('.constraints.','.checkScreenHandlerType.','.checkScreenTitle.')):data[field][key.replace('.chestCleaner.','.autoDeposit.')]=text
    if key.startswith('liquidbounce.module.noRotateSet.mode.resetRotation.rotations'):data[field][key.replace('noRotateSet.mode.resetRotation.rotations','autoDeposit.punchToDeposit.rotations')]=text
  for suffix,text in TEXTS.items():
   key=f'liquidbounce.module.{suffix}.description';data['descriptions'][key]=text[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
