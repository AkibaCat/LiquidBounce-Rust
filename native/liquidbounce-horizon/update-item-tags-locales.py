"""Complete ItemTags Simplified/Traditional Chinese labels and descriptions."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
    'description':('Group dropped items into readable item lists and show their quantities.','将地上的掉落物分组合并，显示物品图标和数量。','將地上的掉落物分組合併，顯示物品圖示與數量。'),
    'filter.description':('Choose whether listed item types are shown or hidden.','选择显示列表中的物品或隐藏列表中的物品。','選擇顯示清單中的物品或隱藏清單中的物品。'),
    'items.description':('Item types used by the filter.','用于筛选掉落物的物品类型列表。','用於篩選掉落物的物品類型清單。'),
    'background.description':('Select a rounded panel or individual vanilla slots.','选择圆角面板背景或原版物品槽背景。','選擇圓角面板背景或原版物品欄背景。'),
    'background.rect.description':('Draw a rounded panel behind the item list.','在物品列表后绘制圆角面板。','在物品清單後繪製圓角面板。'),
    'background.texture.description':('Draw a vanilla container slot behind each item.','在每个物品后绘制原版容器物品槽。','在每個物品後繪製原版容器物品欄。'),
    'background.rect.color.description':('Panel fill color.','面板的填充颜色。','面板的填滿顏色。'),
    'background.rect.outlineColor.description':('Panel outline color; transparent disables the outline.','面板边框颜色；透明色会隐藏边框。','面板邊框顏色；透明色會隱藏邊框。'),
    'background.rect.margin.description':('Space between the item list and its panel edge.','物品列表与面板边缘之间的间距。','物品清單與面板邊緣之間的間距。'),
    'background.rect.radius.description':('Radius of the panel corners.','面板的圆角半径。','面板的圓角半徑。'),
    'background.rect.gradient.description':('How much the panel fill darkens towards its bottom edge.','面板填充色从上至下逐渐变暗的程度。','面板填滿色由上至下逐漸變暗的程度。'),
    'background.rect.glow.description':('Strength of the three outer glow layers.','面板三层外发光的强度。','面板三層外發光的強度。'),
    'scale.description':('Set the item list scale at each camera distance.','通过曲线设置不同相机距离下物品列表的缩放。','透過曲線設定不同攝影機距離下物品清單的縮放。'),
    'renderOffset.description':('Move the tag in world coordinates relative to the cluster center.','相对于掉落物分组中心，按世界坐标偏移标签。','相對於掉落物分組中心，依世界座標偏移標籤。'),
    'rowLength.description':('Maximum number of items in a row.','每行最多显示的物品数量。','每列最多顯示的物品數量。'),
    'preventOverlap.description':('Move overlapping lists down in stable submission order.','按照稳定的显示顺序将重叠列表向下移开。','依照穩定的顯示順序將重疊清單向下移開。'),
    'clusterEntities.description':('Set the grouping radius at each camera distance.','通过曲线设置不同相机距离下掉落物的分组半径。','透過曲線設定不同攝影機距離下掉落物的分組半徑。'),
    'mergeMode.description':('Merge nothing, merge identical item types, or merge identical component patches.','选择不合并、按物品类型合并，或仅合并具有相同组件的物品。','選擇不合併、依物品類型合併，或僅合併具有相同元件的物品。'),
    'shulker.description':('Show nonempty container contents in additional item lists.','为非空的容器物品额外显示其内部物品列表。','為非空的容器物品額外顯示其內部物品清單。'),
    'shulker.enabled.description':('Enable container contents.','启用容器内容显示。','啟用容器內容顯示。'),
    'shulker.mergeStacks.description':('Apply the selected merge mode to container contents.','按选定的合并模式合并容器内部物品。','依選定的合併模式合併容器內部物品。'),
    'shulker.showTitle.description':('Show the container item name above its contents.','在容器内容上方显示该容器物品的名称。','在容器內容上方顯示該容器物品的名稱。'),
}
LABELS={'ClusterEntities':('掉落物分组','掉落物分組'),'MergeMode':('合并模式','合併模式'),'ByItem':('按物品类型','依物品類型'),'ByComponents':('按物品组件','依物品元件'),'PreventOverlap':('避免重叠','避免重疊'),'RowLength':('每行物品数','每列物品數'),'MergeStacks':('合并物品堆叠','合併物品堆疊'),'ShowTitle':('显示标题','顯示標題'),'Shulker':('潜影盒内容','界伏盒內容')}

def update(folder):
    for code,index in [('zh_cn',1),('zh_tw',2)]:
        p=folder/f'display_{code}.json';data=json.loads(p.read_text(encoding='utf8'))
        put_module(data,'liquidbounce.module.itemTags','ItemTags',{'':('掉落物标签','掉落物標籤')[index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
        for key,values in LABELS.items():data['labels'][key]=values[index-1]
        p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
