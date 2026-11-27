"""AutoShoot display strings for the current Minecraft Simplified/Traditional locale."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.autoShoot'
TEXTS={
 'description':('Automatically aim and throw selected items at visible targets.','自动瞄准可见目标并投掷指定物品。','自動瞄準可見目標並投擲指定物品。'),
 'throwableType.description':('Choose eggs and snowballs, or a configurable custom item filter.','选择鸡蛋与雪球，或使用自定义物品筛选。','選擇雞蛋與雪球，或使用自訂物品篩選。'),
 'throwableType.custom.filter.description':('Whitelist accepts listed items; blacklist accepts nonempty items outside the list.','白名单只使用列表中的物品；黑名单使用列表外的非空物品。','白名單只使用列表中的物品；黑名單使用列表外的非空物品。'),
 'throwableType.custom.items.description':('Items matched by the custom throwable filter.','自定义投掷筛选使用的物品列表。','自訂投擲篩選使用的物品清單。'),
 'gravityType.description':('Auto uses projectile gravity for eggs and snowballs and linear aim for other items.','自动模式为鸡蛋与雪球计算抛物线，其余物品使用直线瞄准。','自動模式為雞蛋與雪球計算拋物線，其餘物品使用直線瞄準。'),
 'target.range.description':('Minimum and maximum distance to the target hitbox. Both limits apply.','目标碰撞箱的最小与最大距离；两个边界均会生效。','目標碰撞箱的最小與最大距離；兩個邊界均會生效。'),
 'aimOffThreshold.description':('Maximum angle between the server rotation and the calculated aim before clicking.','服务器朝向与计算瞄准方向的最大夹角；超过时等待转向。','伺服器朝向與計算瞄準方向的最大夾角；超過時等待轉向。'),
 'swingMode.description':('Show the local interaction swing, or hide it on this client.','显示本地交互挥手动画，或仅在本机隐藏。','顯示本機互動揮手動畫，或僅在本機隱藏。'),
 'selectSlotAutomatically.description':('Silently select the nearest matching hotbar slot when it is not already selected.','需要时静默切换至最近的匹配快捷栏槽位。','需要時靜默切換至最近的匹配快捷欄欄位。'),
 'ticksUntilSlotReset.description':('Ticks to retain the silent hotbar selection after a selection request.','请求静默选槽后保持该槽位的游戏刻数。','請求靜默選欄後保持該欄位的遊戲刻數。'),
 'considerInventory.description':('Pause use clicks when an inventory is open and let rotations respect inventory checks.','背包打开时暂停使用点击，并让旋转遵循背包检查。','背包開啟時暫停使用點擊，並讓旋轉遵循背包檢查。'),
 'requiresKillAura.description':('Only operate while KillAura is running; otherwise clear the current target.','仅在杀戮光环运行时工作，否则清除当前目标。','僅在殺戮光環執行時工作，否則清除目前目標。'),
 'notDuringCombat.description':('Pause aiming and use while CombatManager reports combat.','战斗管理器判定正在战斗时暂停瞄准与使用。','戰鬥管理器判定正在戰鬥時暫停瞄準與使用。'),
 'constantLag.description':('Allow FakeLag to retain its constant lag behavior when no shooting target is selected.','没有选中投掷目标时，让假延迟继续保持恒定延迟行为。','沒有選中投擲目標時，讓假延遲繼續保持恆定延遲行為。'),
}
LABELS={'AutoShoot':('自动投掷','自動投擲'),'ThrowableType':('投掷物类型','投擲物類型'),'EggAndSnowball':('鸡蛋与雪球','雞蛋與雪球'),'RequiresKillAura':('需要杀戮光环','需要殺戮光環'),'TicksUntilSlotReset':('选槽复原刻数','選欄復原刻數'),'ConstantLag':('恒定延迟','恆定延遲')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['AutoShoot'][index-1],**{k:v[index]for k,v in TEXTS.items()}}
  put_module(data,PREFIX,'AutoShoot',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  # Shared, source-identical subtrees retain their translated descriptions.
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    if key.startswith('liquidbounce.module.autoRod.')and any(key.startswith('liquidbounce.module.autoRod.'+part)for part in ['aimPoint.','rotations.','targetRendering.','target.']):data[field].setdefault(key.replace('liquidbounce.module.autoRod.',PREFIX+'.'),value)
    if key.startswith('liquidbounce.module.autoClicker.use.clicker.'):
     data[field].setdefault(key.replace('liquidbounce.module.autoClicker.use.clicker.',PREFIX+'.clicker.'),value)
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');args=a.parse_args();update(args.folder)
