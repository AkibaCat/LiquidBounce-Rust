#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={
 'SelectionMode':('收取顺序','收取順序'),'Value':('价值优先','價值優先'),'Distance':('距离优先','距離優先'),'Index':('槽位顺序','欄位順序'),'Random':('随机','隨機'),'StartItem':('起始物品','起始物品'),'Default':('默认','預設'),'MaxSlot':('最大槽位','最大欄位'),'MinSlot':('最小槽位','最小欄位'),'RandomFactor':('距离随机系数','距離隨機係數'),'Order':('顺序','順序'),'Ascending':('升序','升冪'),'Descending':('降序','降冪'),'MoveMode':('移动方式','移動方式'),'QuickMove':('快捷移动','快捷移動'),'DragAndDrop':('拾取放置','拾取放置'),'QuickSwaps':('快捷栏交换','快捷列交換'),'OnFull':('背包满时','背包滿時'),'None':('不操作','不操作'),'Throw':('丢弃无用物品','丟棄無用物品'),
 'Aura':('自动开箱','自動開箱'),'Range':('交互距离','互動距離'),'WallRange':('隔墙距离','隔牆距離'),'Delay':('间隔','間隔'),'SwingMode':('挥手显示','揮手顯示'),'DoNotHide':('正常显示','正常顯示'),'HideForClient':('本地隐藏','本機隱藏'),'NotDuringCombat':('战斗时暂停','戰鬥時暫停'),'TrackManualInteractions':('记录手动交互','記錄手動互動'),'PauseOn':('暂停条件','暫停條件'),'Combat':('战斗中','戰鬥中'),'UsingItem':('使用物品时','使用物品時'),'AwaitContainer':('等待容器打开','等待容器開啟'),'Timeout':('等待超时','等待逾時'),'MaxRetries':('最多重试次数','最多重試次數'),'ValidStorageBlocks':('可交互容器方块','可互動容器方塊'),
 'SilentScreen':('静默界面','靜默介面'),'UnlockCursor':('释放鼠标','釋放滑鼠'),'DrawInventoryTag':('显示容器内容标签','顯示容器內容標籤'),'Background':('背景','背景'),'Rect':('圆角面板','圓角面板'),'Texture':('槽位材质','欄位材質'),'Color':('背景颜色','背景顏色'),'OutlineColor':('描边颜色','描邊顏色'),'Margin':('边距','邊距'),'Radius':('圆角半径','圓角半徑'),'Gradient':('渐变强度','漸層強度'),'Glow':('光晕强度','光暈強度'),'Scale':('缩放','縮放'),'RenderOffset':('标签位置偏移','標籤位置偏移'),'ShowTitle':('显示容器标题','顯示容器標題')}
LABELS.update({'Range':('范围','範圍'),'Color':('颜色','顏色'),'None':('无','無'),'Throw':('丢弃','丟棄'),'RandomFactor':('随机系数','隨機係數'),'SwingMode':('挥手模式','揮手模式')})
TEXTS={
 'description':('Collect useful container items with shared inventory cleanup, optional chest aura and a silent screen.','自动收取容器内有用物品，可配合背包整理、自动开箱和静默界面。','自動收取容器內有用物品，可搭配背包整理、自動開箱與靜默介面。'),
 'selectionMode.description':('Value prioritizes item categories. Distance follows nearby chest slots, Index follows slot numbers and Random shuffles the order.','价值优先按物品类别排序；距离优先收取相邻槽位，槽位顺序按编号排序，随机会打乱顺序。','價值優先依物品類別排序；距離優先收取相鄰欄位，欄位順序依編號排序，隨機會打亂順序。'),
 'selectionMode.distance.startItem.description':('Choose the initial slot before the nearest-neighbor walk. Default keeps the original first slot.','选择距离遍历的起始槽位；默认保留原列表第一个槽位。','選擇距離遍歷的起始欄位；預設保留原清單第一個欄位。'),
 'selectionMode.distance.randomFactor.description':('Multiply each candidate squared slot distance by a separately sampled factor.','为每个候选槽位的平方距离分别乘以随机系数。','為每個候選欄位的平方距離分別乘以隨機係數。'),
 'moveMode.description':('QuickMove uses the container quick-move action. DragAndDrop performs the complete pickup and placement chain, returning excess items.','快捷移动使用容器的快捷转移；拾取放置执行完整点击链，并把放不下的剩余物品放回容器。','快捷移動使用容器的快捷轉移；拾取放置執行完整點擊鏈，並將放不下的剩餘物品放回容器。'),
 'quickSwaps.description':('Use the InventoryCleaner plan to swap container upgrades directly into the hotbar. Useful displaced stacks are preserved in an empty main-inventory slot.','按背包整理计划把容器内升级物品直接换入快捷栏；被换出的有用物品会移到主背包空位。','依背包整理計畫將容器內升級物品直接換入快捷列；被換出的有用物品會移到主背包空位。'),
 'onFull.description':('Throw discards live useless player stacks when more space is required. None stops collecting when no capacity remains.','需要腾出空间时丢弃背包中的无用物品；不操作会在没有容量时停止收取。','需要騰出空間時丟棄背包中的無用物品；不操作會在沒有容量時停止收取。'),
 'aura.description':('Find unvisited storage blocks within range, rotate with the shared rotation service and use the normal main-hand block interaction.','寻找范围内尚未访问的容器，使用共享转向服务瞄准，并执行正常主手方块交互。','尋找範圍內尚未造訪的容器，使用共享轉向服務瞄準，並執行正常主手方塊互動。'),
 'aura.wallRange.description':('Maximum interaction distance through walls. Changing this value limits it to the current Range.','允许隔墙交互的最大距离；修改此值时不会超过当前交互距离。','允許隔牆互動的最大距離；修改此值時不會超過目前互動距離。'),
 'aura.delay.description':('Ticks between successful interactions when AwaitContainer is disabled.','关闭等待容器打开时，成功交互之间的游戏刻间隔。','關閉等待容器開啟時，成功互動之間的遊戲刻間隔。'),
 'aura.swingMode.description':('Show or hide the local interaction swing. The server broadcasts its own successful use animation.','控制本地交互挥手是否可见；服务端按交互结果广播动画。','控制本機互動揮手是否可見；伺服器依互動結果廣播動畫。'),
 'aura.trackManualInteractions.description':('Remember accepted manual use-on-block packets, including the other half of double chests.','记录已接受的手动方块交互，同时排除大型箱子的另一半。','記錄已接受的手動方塊互動，同時排除大型儲物箱的另一半。'),
 'aura.awaitContainer.description':('After a successful interaction, wait for any container screen. Retry only after the timeout; stop after the configured retry count.','成功交互后等待容器界面打开；超时后重试，达到重试次数后跳过该容器。','成功互動後等待容器介面開啟；逾時後重試，達到重試次數後略過該容器。'),
 'aura.validStorageBlocks.description':('Storage block whitelist. Blocked chest halves and previously interacted double chests are excluded.','容器方块白名单；被阻挡的箱子和已交互的大型箱子会被排除。','容器方塊白名單；被阻擋的儲物箱與已互動的大型儲物箱會被排除。'),
 'silentScreen.description':('Hide accepted container screens while keeping the actual menu and automated inventory actions active.','隐藏符合筛选条件的容器界面，同时保留实际菜单与自动物品操作。','隱藏符合篩選條件的容器介面，同時保留實際選單與自動物品操作。'),
 'silentScreen.unlockCursor.description':('Release the mouse when a hidden container opens. Disabled keeps the pointer captured so the view can turn.','静默容器打开时释放鼠标；关闭后保持鼠标捕获，允许转动视角。','靜默容器開啟時釋放滑鼠；關閉後保持滑鼠捕獲，允許轉動視角。'),
 'silentScreen.drawInventoryTag.description':('Show the complete container contents at the last interacted block, centered across both halves of a double chest.','在最后交互的方块位置显示完整容器内容；大型箱子标签位于两半的中心。','在最後互動的方塊位置顯示完整容器內容；大型儲物箱標籤位於兩半的中心。'),
 'silentScreen.drawInventoryTag.background.description':('Rect draws a configurable rounded panel. Texture draws one slot texture for every container slot, including empty slots.','圆角面板可调整颜色和边缘效果；槽位材质为每个容器槽位绘制背景，包括空槽。','圓角面板可調整顏色與邊緣效果；欄位材質為每個容器欄位繪製背景，包括空欄。'),
 'silentScreen.drawInventoryTag.renderOffset.description':('World-space offset applied before projecting the container tag onto the screen.','容器标签投影到屏幕前应用的世界坐标偏移。','容器標籤投影至螢幕前套用的世界座標偏移。')}
for path,en,cn,tw in [
 ('selectionMode.index.order','Order slots by ascending or descending container index.','按容器槽位编号升序或降序收取。','依容器欄位編號升冪或降冪收取。'),
 ('aura.range','Maximum distance to a target storage block.','目标容器的最大交互距离。','目標容器的最大互動距離。'),
 ('aura.notDuringCombat','Pause chest aura while the combat tracker is active.','战斗追踪器激活时暂停自动开箱。','戰鬥追蹤器啟用時暫停自動開箱。'),
 ('aura.pauseOn','Suspend aura for the selected combat or item-use conditions.','在选定的战斗或物品使用条件下暂停自动开箱。','在選定的戰鬥或物品使用條件下暫停自動開箱。'),
 ('aura.awaitContainer.timeout','Ticks to wait for the container screen before a retry.','重试前等待容器界面的游戏刻数。','重試前等待容器介面的遊戲刻數。'),
 ('aura.awaitContainer.maxRetries','Maximum additional retries after an unsuccessful wait.','等待失败后允许追加的最大重试次数。','等待失敗後允許追加的最大重試次數。'),
 ('silentScreen.drawInventoryTag.scale','Scale the whole container tag, including its title and items.','缩放完整容器标签，包括标题与物品。','縮放完整容器標籤，包括標題與物品。'),
 ('silentScreen.drawInventoryTag.showTitle','Display the actual localized container title above its contents.','在内容上方显示实际本地化容器标题。','在內容上方顯示實際本地化容器標題。'),
 ('silentScreen.drawInventoryTag.background.rect.color','ARGB fill color for the rounded background.','圆角背景的 ARGB 填充颜色。','圓角背景的 ARGB 填滿顏色。'),
 ('silentScreen.drawInventoryTag.background.rect.outlineColor','ARGB outline color; fully transparent disables the outline.','ARGB 描边颜色；完全透明时关闭描边。','ARGB 描邊顏色；完全透明時關閉描邊。'),
 ('silentScreen.drawInventoryTag.background.rect.margin','Space around the content before drawing the panel.','面板与内容之间的边距。','面板與內容之間的邊距。'),
 ('silentScreen.drawInventoryTag.background.rect.radius','Rounded-corner radius in unscaled GUI pixels.','未缩放界面像素单位的圆角半径。','未縮放介面像素單位的圓角半徑。'),
 ('silentScreen.drawInventoryTag.background.rect.gradient','Darken the panel fill toward its lower edge.','向下边缘逐渐加深面板填充颜色。','向下邊緣逐漸加深面板填滿顏色。'),
 ('silentScreen.drawInventoryTag.background.rect.glow','Strength of the three outer glow layers.','面板外围三层光晕的强度。','面板外圍三層光暈的強度。')]:TEXTS[path+'.description']=(en,cn,tw)
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));prefix='liquidbounce.module.chestStealer'
  entries={'':('容器收取','容器收取')[index-1]};entries.update({suffix:text[index]for suffix,text in TEXTS.items()})
  put_module(data,prefix,'ChestStealer',entries,{suffix:text[0]for suffix,text in TEXTS.items()});data['labels'].update({name:text[index-1]for name,text in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   for key,text in list(data.get(field,{}).items()):
    if key.startswith('liquidbounce.module.chestCleaner.') and any(tag in key for tag in ['constraints','checkScreen','autoClose']):data[field][key.replace('.chestCleaner.','.chestStealer.')]=text
    if key.startswith('liquidbounce.module.noRotateSet.mode.resetRotation.rotations'):data[field][key.replace('.noRotateSet.mode.resetRotation.rotations','.chestStealer.aura.rotations')]=text
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
