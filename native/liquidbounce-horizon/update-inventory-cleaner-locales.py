#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={
 'Order':('操作顺序','操作順序'),'Value':('价值优先','價值優先'),'Distance':('距离优先','距離優先'),
 'MaximumBlocks':('方块保留量','方塊保留量'),'MaximumArrows':('箭矢保留量','箭矢保留量'),'MaximumThrowables':('投掷物保留量','投擲物保留量'),'MaximumFoodPoints':('食物营养保留量','食物營養保留量'),
 'MaximumWaterBuckets':('水桶保留量','水桶保留量'),'MaximumLavaBuckets':('岩浆桶保留量','熔岩桶保留量'),'MaximumMilkBuckets':('奶桶保留量','奶桶保留量'),'MaximumTotems':('图腾保留量','圖騰保留量'),
 'ItemsBlacklist':('物品黑名单','物品黑名單'),'ThrowUnknownItems':('丢弃未分类物品','丟棄未分類物品'),'Greedy':('贪心选择','貪心選擇'),'OffHandItem':('副手物品','副手物品'),
 'Sword':('剑','劍'),'Weapon':('武器','武器'),'Spear':('长矛','長矛'),'Mace':('重锤','重錘'),'BestKnockback':('最强击退物品','最強擊退物品'),'Bow':('弓','弓'),'Crossbow':('弩','弩'),
 'Axe':('斧','斧'),'Pickaxe':('镐','鎬'),'Shovel':('锹','鏟'),'Hoe':('锄','鋤'),'Rod':('钓鱼竿','釣魚竿'),'Shield':('盾牌','盾牌'),'Water':('水桶','水桶'),'Lava':('岩浆桶','熔岩桶'),'Milk':('奶桶','奶桶'),
 'Pearl':('末影珍珠','終界珍珠'),'Gapple':('金苹果','金蘋果'),'Food':('食物','食物'),'Potion':('药水','藥水'),'Block':('方块','方塊'),'Throwables':('投掷物','投擲物'),'Ignore':('忽略','忽略'),'None':('不指定','不指定')}
TEXTS={
 'description':('Sort inventory items, merge partial stacks and discard items outside the cleanup plan.','按价值整理物品栏、合并零散堆叠并丢弃整理计划不保留的物品。','依價值整理物品欄、合併零散堆疊並丟棄整理計畫不保留的物品。'),
 'order.description':('Value sorts swaps by category priority. Distance follows the inventory manager cursor using inventory slot positions.','价值优先按类别优先级交换；距离优先沿库存管理器光标附近的物品栏槽位依次操作。','價值優先依類別優先序交換；距離優先沿庫存管理器游標附近的物品欄欄位依次操作。'),
 'itemsBlacklist.description':('Never retain listed items, except items currently locked by the server.','丢弃黑名单中的物品；服务端锁定的物品仍受保护。','丟棄黑名單中的物品；伺服器鎖定的物品仍受保護。'),
 'throwUnknownItems.description':('Discard unclassified items. Custom-named server items keep a separate protected category.','丢弃未分类物品；带自定义名称的服务器物品使用独立类别，继续保留。','丟棄未分類物品；帶自訂名稱的伺服器物品使用獨立類別，繼續保留。'),
 'greedy.description':('Preserved upstream option. The pinned cleanup generator does not implement its greedy check.','保留原版选项；当前固定版本的整理生成器尚未实现贪心检查。','保留原版選項；目前固定版本的整理產生器尚未實作貪心檢查。'),
 'offHandItem.description':('Preferred category in the offhand. Ignore keeps its current item. Offhand automation and server locks take precedence.','指定副手类别；忽略会保留当前物品。副手自动化和服务端锁定具有优先权。','指定副手類別；忽略會保留目前物品。副手自動化與伺服器鎖定具有優先權。'),
}
for name in ['MaximumBlocks','MaximumArrows','MaximumThrowables','MaximumFoodPoints','MaximumWaterBuckets','MaximumLavaBuckets','MaximumMilkBuckets','MaximumTotems']:
 suffix=name[0].lower()+name[1:]+'.description'
 TEXTS[suffix]=('Desired retained amount. Whole stacks are accepted until the amount and assigned hotbar slots are satisfied; a stack may pass the target. Food is measured in nutrition points.','期望保留量。按完整堆叠保留，直到数量和指定快捷栏槽位均已满足；最后一组可超过目标。食物按营养点数计算。','期望保留量。依完整堆疊保留，直到數量與指定快捷列欄位皆已滿足；最後一組可超過目標。食物依營養點數計算。')
for i in range(1,10):
 LABELS[f'SlotItem-{i}']=(f'快捷栏 {i}',f'快捷列 {i}')
 TEXTS[f'slotItem-{i}.description']=('Preferred category for this hotbar slot. Ignore protects its current item; None does not reserve a category.','此快捷栏槽位的首选类别；忽略保护当前物品，不指定则不预留类别。','此快捷列欄位的首選類別；忽略保護目前物品，不指定則不預留類別。')
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));prefix='liquidbounce.module.inventoryCleaner'
  entries={'':('背包整理','背包整理')[index-1]};entries.update({suffix:text[index]for suffix,text in TEXTS.items()})
  put_module(data,prefix,'InventoryCleaner',entries,{suffix:text[0]for suffix,text in TEXTS.items()})
  data['labels'].update({name:text[index-1]for name,text in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   if field in data:
    for key,text in list(data[field].items()):
     if key.startswith('liquidbounce.module.elytraSwap.constraints'):
      data[field][key.replace('.elytraSwap.','.inventoryCleaner.')]=text
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
