#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={
 'AutoArmor':('自动护甲','自動護甲'),'Smart':('智能','智慧'),'RawDefense':('基础防御','基礎防禦'),
 'ConsiderProjectileProtection':('考虑弹射物保护','考慮投射物保護'),'Hotbar':('快捷栏','快捷列'),
 'CanSwapArmor':('允许直接替换护甲','允許直接替換護甲'),'SaveArmor':('保护低耐久护甲','保護低耐久護甲'),
 'DurabilityThreshold':('耐久阈值','耐久門檻'),'AutoOpenInventory':('自动打开背包','自動開啟背包'),
 'SkipCursed':('排除诅咒护甲','排除詛咒護甲'),'Binding':('绑定诅咒','綁定詛咒'),'Vanishing':('消失诅咒','消失詛咒')}
TEXTS={
 'description':('Equip the best player armor with shared inventory constraints, hotbar use and low-durability protection.','自动穿戴最佳玩家护甲，支持快捷栏穿戴、库存操作限制和低耐久保护。','自動穿戴最佳玩家護甲，支援快捷列穿戴、物品欄操作限制與低耐久保護。'),
 'mode.description':('Smart evaluates protection enchantments and toughness. RawDefense ranks real reduction under the legacy armor formula.','智能模式综合保护附魔与韧性；基础防御模式按旧版护甲公式的实际减伤评分。','智慧模式綜合保護附魔與韌性；基礎防禦模式依舊版護甲公式的實際減傷評分。'),
 'mode.rawDefense.considerProjectileProtection.description':('Include Projectile Protection in the primary RawDefense reduction score.','在基础防御模式的主要减伤评分中计入弹射物保护。','在基礎防禦模式的主要減傷評分中計入投射物保護。'),
 'hotbar.description':('Use armor directly from the hotbar or offhand when the player inventory is closed.','背包关闭时直接使用快捷栏或副手中的护甲。','背包關閉時直接使用快捷列或副手中的護甲。'),
 'hotbar.canSwapArmor.description':('Allow item use to swap occupied armor slots on versions supporting armor swapping.','允许通过使用物品直接替换已穿戴的护甲。','允許透過使用物品直接替換已穿戴的護甲。'),
 'saveArmor.description':('Prefer replacement armor above the remaining-durability threshold.','优先穿戴剩余耐久高于阈值的替代护甲。','優先穿戴剩餘耐久高於門檻的替代護甲。'),
 'saveArmor.durabilityThreshold.description':('Armor at or below this remaining durability is replaced when a suitable spare exists.','存在合适备用护甲时，替换剩余耐久小于或等于该值的护甲。','存在合適備用護甲時，替換剩餘耐久小於或等於此值的護甲。'),
 'saveArmor.autoOpenInventory.description':('When InventoryOpen is required, open the inventory to save fragile armor and close it after replacement. A broken piece while a container is open closes that container to refresh armor.','要求背包打开时，自动打开背包以保护濒损护甲，替换后自动关闭；容器打开期间护甲损坏时会关闭容器以刷新护甲。','要求背包開啟時，自動開啟背包以保護瀕損護甲，替換後自動關閉；容器開啟期間護甲損壞時會關閉容器以重新同步護甲。'),
 'skipCursed.description':('Never equip spare armor with a selected curse. Already worn armor remains part of the comparison, including server-locked pieces.','不穿戴带有所选诅咒的备用护甲；已穿戴护甲仍参与比较，包括服务端锁定的护甲。','不穿戴帶有所選詛咒的備用護甲；已穿戴護甲仍參與比較，包括伺服器鎖定的護甲。')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoArmor'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.autoArmor','AutoArmor',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    if key.startswith('liquidbounce.module.elytraSwap.constraints'):
     data[field][key.replace('.elytraSwap.','.autoArmor.')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
