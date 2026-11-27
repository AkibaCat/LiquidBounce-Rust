"""Complete BedPlates Simplified/Traditional Chinese settings and notifications."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
BASE=runpy.run_path(str(HERE/'update-item-tags-locales.py'))['TEXTS']
TEXTS={k:BASE[k]for k in ['background.description','background.rect.description','background.texture.description','background.rect.color.description','background.rect.outlineColor.description','background.rect.margin.description','background.rect.radius.description','background.rect.gradient.description','background.rect.glow.description','scale.description','preventOverlap.description']}
TEXTS.update({
 'description':('Display beds and their surrounding defense blocks.','显示床的位置和周围防御方块。','顯示床的位置和周圍防禦方塊。'),
 'outline.description':('Use the bed dye map color for the panel outline.','使用床的染料颜色绘制面板边框。','使用床的染料顏色繪製面板邊框。'),
 'maxLayers.description':('Maximum number of defense layers scanned around both bed halves.','扫描床两部分周围防御层的最大层数。','掃描床兩部分周圍防禦層的最大層數。'),
 'showBed.description':('Include the bed item and its distance before defense blocks.','在防御方块前显示床的图标和距离。','在防禦方塊前顯示床的圖示與距離。'),
 'textShadow.description':('Draw a shadow behind defense layer numbers and block counts.','为防御层数和方块数量文字绘制阴影。','為防禦層數與方塊數量文字繪製陰影。'),
 'renderOffset.description':('Offset bed labels in world coordinates.','按世界坐标偏移床标签。','依世界座標偏移床標籤。'),
 'maxCount.description':('Maximum number of visible bed lists.','最多同时显示的床标签数量。','最多同時顯示的床標籤數量。'),
 'highlightUnbreakable.description':('Show a red count when no hotbar item can correctly harvest a defense block.','快捷栏中没有能正确采集防御方块的工具时，以红色显示数量。','快捷列中沒有能正確採集防禦方塊的工具時，以紅色顯示數量。'),
 'compact.description':('Combine matching defense blocks across layers.','合并不同防御层中的同种方块。','合併不同防禦層中的同種方塊。'),
 'filterMode.description':('Select the predefined defense filter or a custom block list.','选择预设的防御方块筛选规则或自定义方块列表。','選擇預設的防禦方塊篩選規則或自訂方塊清單。'),
 'filterMode.predefined.description':('Show redstone conductors, ladders, water and full glass blocks.','显示红石导体、梯子、水和完整玻璃方块。','顯示紅石導體、梯子、水和完整玻璃方塊。'),
 'filterMode.custom.description':('Filter defense blocks using your own registry list.','使用自定义注册表列表筛选防御方块。','使用自訂登錄表清單篩選防禦方塊。'),
 'filterMode.custom.blocks.description':('Defense block types used by the custom filter.','自定义筛选使用的防御方块类型。','自訂篩選使用的防禦方塊類型。'),
 'filterMode.custom.filter.description':('Choose whether the listed defense blocks are shown or hidden.','选择显示列表中的防御方块或隐藏列表中的防御方块。','選擇顯示清單中的防禦方塊或隱藏清單中的防禦方塊。'),
 'ignoreSelfBed.description':('Choose how to identify and hide your own bed.','选择识别并隐藏自身床的方式。','選擇識別並隱藏自身床的方式。'),
 'ignoreSelfBed.none.description':('Show every bed without identifying your own bed.','显示所有床，不识别自身床。','顯示所有床，不識別自身床。'),
 'ignoreSelfBed.color.description':('Match the bed dye against selected dyed armor pieces.','根据所选染色盔甲的颜色识别自身床。','依所選染色盔甲的顏色識別自身床。'),
 'ignoreSelfBed.color.slots.description':('Armor slots whose dyed color is checked.','用于检查染色颜色的盔甲栏位。','用於檢查染色顏色的盔甲欄位。'),
 'ignoreSelfBed.color.loose.description':('Match the nearest dye color instead of requiring an exact armor RGB match.','按最接近的染料色匹配，无需盔甲 RGB 完全一致。','依最接近的染料色比對，無需盔甲 RGB 完全一致。'),
 'ignoreSelfBed.spawnLocation.description':('Identify your bed near the position received at game start.','根据游戏开始时收到的位置识别附近的自身床。','依遊戲開始時收到的位置識別附近的自身床。'),
 'ignoreSelfBed.spawnLocation.bedDistance.description':('Maximum distance from the tracked spawn position to your bed.','记录的出生位置与自身床之间的最大距离。','記錄的出生位置與自身床之間的最大距離。'),
 'ignoreSelfBed.manual.description':('Mark a nearby bed using configurable keyboard keys.','通过可配置的键盘按键标记附近的床。','透過可設定的鍵盤按鍵標記附近的床。'),
 'ignoreSelfBed.manual.track.description':('Keyboard key that tracks the nearest bed in the nearby cube.','将附近立方区域内最近的床设为自身床的键盘按键。','將附近立方區域內最近的床設為自身床的鍵盤按鍵。'),
 'ignoreSelfBed.manual.untrack.description':('Keyboard key that clears the manually tracked bed.','取消手动记录的自身床的键盘按键。','取消手動記錄的自身床的鍵盤按鍵。'),
 'ignoreAdjacent.description':('Hide beds whose tracked head touches another bed head.','隐藏床头与另一个床头相邻的床。','隱藏床頭與另一個床頭相鄰的床。'),
 'notifications.title':('SelfBed-Manual','自身床－手动','自身床－手動'),
 'notifications.missing':('Cannot find any bed around you! Please get close to your bed.','附近没有找到床！请靠近你的床。','附近沒有找到床！請靠近你的床。'),
 'notifications.tracked':('Tracked bed position ({x}, {y}, {z}).','已记录床的位置（{x}, {y}, {z}）。','已記錄床的位置（{x}, {y}, {z}）。'),
 'notifications.untracked':('Bed position ({x}, {y}, {z}) has been untracked.','已取消记录床的位置（{x}, {y}, {z}）。','已取消記錄床的位置（{x}, {y}, {z}）。'),
 'notifications.worldChange':('Bed position ({x}, {y}, {z}) has been untracked due to world change.','世界已切换，已取消记录床的位置（{x}, {y}, {z}）。','世界已切換，已取消記錄床的位置（{x}, {y}, {z}）。'),
})
LABELS={'Outline':('边框','邊框'),'MaxLayers':('最大层数','最大層數'),'ShowBed':('显示床','顯示床'),'TextShadow':('文字阴影','文字陰影'),'MaxCount':('最大显示数量','最大顯示數量'),'HighlightUnbreakable':('标记无法采集的方块','標記無法採集的方塊'),'Compact':('紧凑显示','緊湊顯示'),'FilterMode':('筛选模式','篩選模式'),'Predefined':('预设','預設'),'Custom':('自定义','自訂'),'IgnoreSelfBed':('隐藏自身床','隱藏自身床'),'Color':('颜色','顏色'),'Slots':('槽位','欄位'),'Feet':('脚部','腳部'),'Legs':('腿部','腿部'),'Chest':('胸部','胸部'),'Head':('头部','頭部'),'Loose':('近似颜色匹配','近似顏色比對'),'SpawnLocation':('出生位置','出生位置'),'BedDistance':('床判定距离','床判定距離'),'Manual':('手动','手動'),'Track':('记录','記錄'),'Untrack':('取消记录','取消記錄'),'IgnoreAdjacent':('忽略相邻床','忽略相鄰床')}
def update(folder):
    for code,index in [('zh_cn',1),('zh_tw',2)]:
        p=folder/f'display_{code}.json';data=json.loads(p.read_text(encoding='utf8'))
        put_module(data,'liquidbounce.module.bedPlates','BedPlates',{'':('床防御标签','床防禦標籤')[index-1],**{k:v[index]for k,v in TEXTS.items()if not k.startswith('notifications.')}},{k:v[0]for k,v in TEXTS.items()})
        for key,values in TEXTS.items():
            if key.startswith('notifications.'):data['descriptions']['liquidbounce.module.bedPlates.'+key]=values[index]
        for key,values in LABELS.items():
            if key!='Chest':data['labels'][key]=values[index-1]
        data['labels']['liquidbounce.module.bedPlates.ignoreSelfBed.color.slots.Chest']=LABELS['Chest'][index-1]
        p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
