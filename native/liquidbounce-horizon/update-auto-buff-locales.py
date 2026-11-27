#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={
 'AutoBuff':('自动补给','自動補給'),'Soup':('蘑菇煲','蘑菇湯'),'Head':('玩家头颅','玩家頭顱'),'Pot':('投掷药水','投擲藥水'),'Drink':('饮用药水','飲用藥水'),'Gapple':('金苹果','金蘋果'),
 'AfterUse':('使用后处理','使用後處理'),'AssumeEmptyBowl':('假定已变为空碗','假定已變為空碗'),'IgnoreInventoryFull':('忽略背包已满','忽略背包已滿'),'ConsiderAbsorption':('计入伤害吸收','計入傷害吸收'),'MaxAbsorption':('最大伤害吸收','最大傷害吸收'),'Enchanted':('允许附魔金苹果','允許附魔金蘋果'),
 'AutoSwap':('自动换槽','自動換欄位'),'DelayIn':('使用前延迟','使用前延遲'),'DelayOut':('使用后延迟','使用後延遲'),'Refill':('补充快捷栏','補充快捷列'),'Potions':('药效选择','藥效選擇'),'Regen':('生命恢复','生命恢復'),'FireResistance':('抗火','抗火'),'JumpBoost':('跳跃提升','跳躍提升'),'WaterBreathing':('水下呼吸','水下呼吸'),
 'TillGroundDistance':('最大离地距离','最大離地距離'),'DoNotBenefitOthers':('避免增益敌人','避免增益敵人'),'AllowLingering':('允许滞留药水','允許滯留藥水'),'RotationTiming':('旋转时机','旋轉時機'),'Normal':('正常','正常'),'OnTick':('移动时发送','移動時傳送'),'OnUse':('使用时发送','使用時傳送'),
 'NotDuringCombat':('战斗期间停用','戰鬥期間停用'),'IgnoreOpenInventory':('忽略背包打开状态','忽略背包開啟狀態')}
TEXTS={}
def text(path,en,cn,tw):TEXTS[path+'.description' if path else 'description']=(en,cn,tw)
text('','Automatically use healing food and beneficial potions, refill the hotbar, and manage empty soup bowls.','自动使用恢复类食物与增益药水，补充快捷栏并处理蘑菇煲空碗。','自動使用恢復類食物與增益藥水，補充快捷列並處理蘑菇湯空碗。')
for feature,en,cn,tw in [('soup','mushroom stew','蘑菇煲','蘑菇湯'),('head','player heads','玩家头颅','玩家頭顱'),('pot','splash potions','喷溅药水','飛濺藥水'),('drink','drinkable potions','可饮用药水','可飲用藥水'),('gapple','golden apples','金苹果','金蘋果')]:
 text(feature,f'Configure automatic use of {en}.',f'配置自动使用{cn}。',f'設定自動使用{tw}。')
 text(feature+'.enabled',f'Allow AutoBuff to use {en}.',f'允许自动补给使用{cn}。',f'允許自動補給使用{tw}。')
for feature in ['soup','head','gapple']:
 text(feature+'.health','Use this item when health is at or below this percentage of maximum health.','生命值不高于最大生命值的此百分比时使用。','生命值不高於最大生命值的此百分比時使用。')
 text(feature+'.considerAbsorption','Include absorption when testing the health threshold.','检查生命阈值时计入伤害吸收生命值。','檢查生命門檻時計入傷害吸收生命值。')
text('head.maxAbsorption','Only use a head while absorption is at or below this amount.','伤害吸收生命值不高于此数值时才使用头颅。','傷害吸收生命值不高於此數值時才使用頭顱。')
text('head.cooldown','Minimum elapsed seconds between head uses.','两次使用玩家头颅之间的最短秒数。','兩次使用玩家頭顱之間的最短秒數。')
text('gapple.enchanted','Allow enchanted golden apples in addition to ordinary golden apples.','除普通金苹果外，也允许使用附魔金苹果。','除普通金蘋果外，也允許使用附魔金蘋果。')
text('soup.afterUse','Choose whether to keep, drop, or move the empty bowl after using soup.','选择喝汤后保留、丢弃或移动空碗。','選擇喝湯後保留、丟棄或移動空碗。')
for mode in ['drop','stack']:
 text(f'soup.afterUse.{mode}.assumeEmptyBowl','Assume the soup became a bowl without waiting for the server slot update.','无需等待服务器槽位更新，直接假定蘑菇煲已变为空碗。','無需等待伺服器欄位更新，直接假定蘑菇湯已變為空碗。')
 text(f'soup.afterUse.{mode}.wait','Random ticks to wait after using soup before handling its bowl.','使用蘑菇煲后，随机等待这些游戏刻再处理空碗。','使用蘑菇湯後，隨機等待這些遊戲刻再處理空碗。')
text('soup.afterUse.stack.ignoreInventoryFull','Attempt to quick-move the bowl even when no inventory slot is empty.','即使背包没有空槽，也尝试快速移动空碗以合并堆叠。','即使背包沒有空欄位，也嘗試快速移動空碗以合併堆疊。')
effects=[('health','instant health','瞬间治疗','瞬間治療'),('regen','regeneration','生命恢复','生命恢復'),('strength','strength','力量','力量'),('speed','speed','迅捷','迅捷'),('fireResistance','fire resistance','抗火','抗火'),('jumpBoost','jump boost','跳跃提升','跳躍提升'),('waterBreathing','water breathing','水下呼吸','水下呼吸')]
for feature in ['pot','drink']:
 text(feature+'.potions','Choose beneficial potion effects to apply when they are not already active.','选择尚未生效时需要补充的增益药效。','選擇尚未生效時需要補充的增益藥效。')
 for effect,en,cn,tw in effects:
  text(f'{feature}.potions.{effect}',f'Configure {en} potions.',f'配置{cn}药水。',f'設定{tw}藥水。')
  text(f'{feature}.potions.{effect}.enabled',f'Use a potion containing {en} when the effect is absent.',f'没有{cn}效果时允许使用包含此药效的药水。',f'沒有{tw}效果時允許使用包含此藥效的藥水。')
  if effect in ['health','regen']:text(f'{feature}.potions.{effect}.health','Only use this effect at or below this health percentage, excluding absorption.','生命值不高于此百分比时才使用此药效；不计入伤害吸收。','生命值不高於此百分比時才使用此藥效；不計入傷害吸收。')
text('pot.tillGroundDistance','Throw only when the predicted landing block is within this vertical distance.','预测落地的方块与玩家的垂直距离不超过此值时投掷。','預測落地的方塊與玩家的垂直距離不超過此值時投擲。')
text('pot.doNotBenefitOthers','Avoid throwing while a potion-susceptible enemy is within four blocks.','四格内有可受到药水影响的敌人时避免投掷增益药水。','四格內有可受到藥水影響的敵人時避免投擲增益藥水。')
text('pot.allowLingering','Allow lingering potions as well as splash potions.','除喷溅药水外，也允许投掷滞留药水。','除飛濺藥水外，也允許投擲滯留藥水。')
text('autoSwap','Silently select the closest valid hotbar slot when the item is not selected.','需要的物品未选中时，静默切换到最近的有效快捷栏槽位。','需要的物品未選中時，靜默切換至最近的有效快捷列欄位。')
text('autoSwap.enabled','Allow automatic hotbar switching before using a buff.','允许使用补给前自动切换快捷栏槽位。','允許使用補給前自動切換快捷列欄位。')
text('autoSwap.delayIn','Random ticks to wait after selecting a buff slot.','选中补给槽位后，随机等待这些游戏刻再使用。','選中補給欄位後，隨機等待這些遊戲刻再使用。')
text('autoSwap.delayOut','Random ticks added after a switched-slot buff before another buff and slot restoration.','换槽使用补给后，随机等待这些游戏刻再进行下一次补给并恢复槽位。','換欄位使用補給後，隨機等待這些遊戲刻再進行下一次補給並恢復欄位。')
text('refill','Quick-move useful buffs from the inventory into empty or mergeable hotbar slots.','将背包内可用的补给快速移动到快捷栏空槽，或合并到已有堆叠。','將背包內可用的補給快速移動至快捷列空欄位，或合併至已有堆疊。')
text('refill.enabled','Refill the hotbar with enabled buff items.','使用已启用补给类型的物品补充快捷栏。','使用已啟用補給類型的物品補充快捷列。')
text('rotations.rotationTiming','Normal waits for managed aim; OnTick sends full movement packets; OnUse only aims the item-use packet.','正常：等待转头完成；移动时发送：用完整移动包临时转向；使用时发送：仅在物品使用包中瞄准。','正常：等待轉頭完成；移動時傳送：用完整移動封包暫時轉向；使用時傳送：僅在物品使用封包中瞄準。')
text('combatPauseTime','Pause automatic combat for at least this many ticks when a buff item is selected.','选定补给物品时，至少暂停自动战斗这些游戏刻。','選定補給物品時，至少暫停自動戰鬥這些遊戲刻。')
text('notDuringCombat','Do not start a new buff while the combat manager reports combat.','战斗管理器判定处于战斗时，不开始新的自动补给。','戰鬥管理器判定處於戰鬥時，不開始新的自動補給。')
text('ignoreOpenInventory','Allow buff use while the player inventory is open locally or server-side.','玩家背包在本地或服务器端打开时仍允许使用补给。','玩家背包在本機或伺服器端開啟時仍允許使用補給。')
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoBuff'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.autoBuff','AutoBuff',entries,{k:v[0]for k,v in TEXTS.items()});data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
    if key.startswith(old):data[field][key.replace(old,'liquidbounce.module.autoBuff.rotations')]=value
    old='liquidbounce.module.elytraSwap.constraints'
    if key.startswith(old):
     for target in ['soup.afterUse.stack.constraints','refill.constraints']:data[field][key.replace(old,'liquidbounce.module.autoBuff.'+target)]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
