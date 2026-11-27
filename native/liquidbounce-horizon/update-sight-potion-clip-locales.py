from pathlib import Path
import argparse,json,re
ROOT=Path(__file__).resolve().parents[2]
HELP={
'en_us':{'trueSight.sight':'Objects to reveal: invisible living entities, barriers and light blocks. Markers use the normal random animate-tick sampling.','trueSight.entityColor':'ARGB tint of invisible living entity bodies. Default white, alpha 100.','trueSight.entityFeatureLayerColor':'ARGB tint of colored cutout feature models. Equipment and held items keep their own render colors. Default white, alpha 120.','noClip.speed':'Horizontal and vertical flight speed. Jump takes priority over sneak. Default 0.32; UI range 0.1–0.4.','noClip.onlyInVehicle':'Pause NoClip while you do not control a vehicle. Pausing restores collision without disabling the module.','noClip.disableOnSetback':'Show the original setback error and disable NoClip when a player-position correction arrives while active.','potionSpoof.level':'Spoofed potion level. Default 1; UI range 1–10. Imported levels retain their raw value; the effect amplifier is clamped to 0–255 by Minecraft.','potionSpoof.enabled':'Add this zero-duration effect and its attribute modifiers each player tick. Turning it off removes only a zero-duration effect.'},
'zh_cn':{'trueSight.sight':'选择要显示的对象：隐形生物、屏障和光源方块。方块标记沿用游戏随机动画 tick 的采样方式。','trueSight.entityColor':'隐形生物本体的 ARGB 颜色。默认白色，不透明度 100／255。','trueSight.entityFeatureLayerColor':'彩色镂空附加模型的 ARGB 颜色；装备和手持物品保留自己的渲染颜色。默认白色，不透明度 120／255。','noClip.speed':'水平与垂直飞行速度，同时按跳跃与潜行时优先上升。默认 0.32，界面范围 0.1～0.4。','noClip.onlyInVehicle':'未控制载具时暂停穿墙，恢复碰撞但保持模块开启。','noClip.disableOnSetback':'在实际生效时收到玩家位置修正包，显示原版回弹提示并关闭穿墙。','potionSpoof.level':'伪装的药水等级，默认 1，界面范围 1～10。导入等级保留原始值，Minecraft 将效果强度限制为 0～255。','potionSpoof.enabled':'每个玩家 tick 添加持续时间为零的此效果及属性修饰器；关闭分组时只移除持续时间为零的效果。'},
'zh_tw':{'trueSight.sight':'選擇要顯示的對象：隱形生物、屏障和光源方塊。方塊標記沿用遊戲隨機動畫 tick 的取樣方式。','trueSight.entityColor':'隱形生物本體的 ARGB 顏色。預設白色，不透明度 100／255。','trueSight.entityFeatureLayerColor':'彩色鏤空附加模型的 ARGB 顏色；裝備和手持物品保留自己的算繪顏色。預設白色，不透明度 120／255。','noClip.speed':'水平與垂直飛行速度，同時按跳躍與潛行時優先上升。預設 0.32，介面範圍 0.1～0.4。','noClip.onlyInVehicle':'未控制載具時暫停穿牆，恢復碰撞但保持模組開啟。','noClip.disableOnSetback':'在實際生效時收到玩家位置修正封包，顯示原版回彈提示並關閉穿牆。','potionSpoof.level':'偽裝的藥水等級，預設 1，介面範圍 1～10。匯入等級保留原始值，Minecraft 將效果強度限制為 0～255。','potionSpoof.enabled':'每個玩家 tick 加入持續時間為零的此效果及屬性修飾器；關閉群組時只移除持續時間為零的效果。'}}
LABELS={'zh_cn':{'TrueSight':'真实视界','Sight':'显示对象','Barriers':'屏障','Lights':'光源','Entities':'实体','EntityColor':'实体颜色','EntityFeatureLayerColor':'实体附加层颜色','NoClip':'穿墙','Speed':'速度','OnlyInVehicle':'仅载具内','DisableOnSetback':'回弹时关闭','PotionSpoof':'药水伪装','Level':'等级'},'zh_tw':{'TrueSight':'真實視界','Sight':'顯示對象','Barriers':'屏障','Lights':'光源','Entities':'實體','EntityColor':'實體顏色','EntityFeatureLayerColor':'實體附加層顏色','NoClip':'穿牆','Speed':'速度','OnlyInVehicle':'僅載具內','DisableOnSetback':'回彈時關閉','PotionSpoof':'藥水偽裝','Level':'等級'}}
def update(language,target):
 data=json.loads(target.read_text(encoding='utf8'));help={f'liquidbounce.module.{key}.description':value for key,value in HELP[language].items()}
 source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 for module in ['trueSight','potionSpoof','noClip']:
  key=f'liquidbounce.module.{module}.description';help[key]=source[key]
 english=json.loads((ROOT/'build/resources/26.3/assets/minecraft/lang/en_us.json').read_text(encoding='utf8'));translated=json.loads((ROOT/f'build/resources/26.3/assets/minecraft/lang/{language}.json').read_text(encoding='utf8'))
 table=(ROOT/'native/liquidbounce-behaviors/src/potion_spoof_effects.rs').read_text(encoding='utf8');ids=re.findall(r'\("([a-z_]+)"',table)
 labels=LABELS[language].copy()
 for id in ids:
  key=f'liquidbounce.module.potionSpoof.{id}';label=translated.get('effect.minecraft.'+id,'Unknown');labels[english.get('effect.minecraft.'+id,'Unknown')]=label
  help[key+'.description']=label+' — '+HELP[language]['potionSpoof.enabled'];help[key+'.enabled.description']=HELP[language]['potionSpoof.enabled'];help[key+'.level.description']=HELP[language]['potionSpoof.level']
 data['descriptions'].update(help);data['labels'].update(labels)
 if language=='zh_cn':
  en=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  data['englishDescriptions'].update({f'liquidbounce.module.{k}.description':v for k,v in HELP['en_us'].items()})
  for module in ['trueSight','potionSpoof','noClip']:
   key=f'liquidbounce.module.{module}.description';data['englishDescriptions'][key]=en[key]
  for id in ids:
   key=f'liquidbounce.module.potionSpoof.{id}';data['englishDescriptions'][key+'.description']=HELP['en_us']['potionSpoof.enabled'];data['englishDescriptions'][key+'.enabled.description']=HELP['en_us']['potionSpoof.enabled'];data['englishDescriptions'][key+'.level.description']=HELP['en_us']['potionSpoof.level']
 target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);a=p.parse_args()
 for language in ['zh_cn','zh_tw']:update(language,ROOT/a.directory/f'display_{language}.json')
