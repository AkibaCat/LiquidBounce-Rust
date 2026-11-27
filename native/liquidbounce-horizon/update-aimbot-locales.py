"""Aimbot complete simplified/traditional display localization; stable IDs."""
from pathlib import Path
import argparse,json,runpy
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
PREFIX='liquidbounce.module.aimbot'
LABELS={
 'Aimbot':('自动瞄准','自動瞄準'),'AimAssist':('辅助瞄准','輔助瞄準'),'AutoAim':('自动瞄准','自動瞄準'),
 'WallRange':('穿墙距离','穿牆距離'),'LazyRotation':('保留已对准朝向','保留已對準朝向'),'Crystals':('末影水晶','終界水晶'),
 'Only':('仅瞄准水晶','僅瞄準水晶'),'OnlyWhileUsing':('仅按住使用键时','僅按住使用鍵時'),'EmptyHand':('主手为空','主手為空'),
 'AngleSmooth':('转向平滑','轉向平滑'),'Interpolation':('插值平滑','插值平滑'),'Sigmoid':('S 形平滑','S 形平滑'),
 'HorizontalSpeed':('水平插值速度','水平插值速度'),'VerticalSpeed':('垂直插值速度','垂直插值速度'),'DirectionChangeFactor':('方向变化系数','方向變化係數'),
 'Midpoint':('中点','中點'),'HorizontalTurnSpeed':('水平转向速度','水平轉向速度'),'VerticalTurnSpeed':('垂直转向速度','垂直轉向速度'),
 'Steepness':('曲线陡峭度','曲線陡峭度'),'Axis':('瞄准轴','瞄準軸'),'Horizontal':('水平','水平'),'Vertical':('垂直','垂直'),
 'Screen':('界面','介面'),'Container':('容器','容器'),
}
TEXTS={}
def text(path,en,cn,tw):TEXTS[(path+'.description')if path else'description']=(en,cn,tw)
text('','Automatically face selected living targets or end crystals using mouse-aware interpolated rotation.','自动转向选定的生物目标或末影水晶，并通过插值与鼠标输入保持平滑。','自動轉向選定的生物目標或終界水晶，並透過插值與滑鼠輸入保持平滑。')
text('range','Maximum distance from the player eyes to the nearest target hitbox point.','玩家眼睛到目标碰撞箱最近点的最大距离。','玩家眼睛到目標碰撞箱最近點的最大距離。')
text('wallRange','Allowed hidden-hit distance; changing this value clamps it to the current Range.','允许穿墙瞄准的距离；更改此值时，将其限制为当前作用距离以内。','允許穿牆瞄準的距離；變更此值時，將其限制為目前作用距離以內。')
text('lazyRotation','Retain the current player direction when its entity ray already hits the chosen target within visibility limits.','当前实体射线已经在可见性与距离限制内命中目标时，保留玩家朝向。','目前實體射線已經在可見性與距離限制內命中目標時，保留玩家朝向。')
text('crystals','Consider end crystals first, sorted by the smallest crosshair-to-hitbox angle.','优先选择末影水晶，并按准星与碰撞箱之间的最小夹角排序。','優先選擇終界水晶，並按準星與碰撞箱之間的最小夾角排序。')
text('crystals.enabled','Enable crystal candidates before the ordinary target list.','在普通目标列表之前加入水晶候选目标。','在一般目標清單之前加入水晶候選目標。')
text('crystals.only','Exclude living targets while crystal targeting is enabled, including when no crystal qualifies.','水晶瞄准启用时排除生物目标；即使没有合适的水晶也不改选生物。','水晶瞄準啟用時排除生物目標；即使沒有合適的水晶也不改選生物。')
text('crystals.onlyWhileUsing','Include crystals only while the physical use-item binding is held.','仅在实际按住物品使用键时选择水晶。','僅在實際按住物品使用鍵時選擇水晶。')
text('requires','Require every selected condition; any open screen prevents preparing a new aim regardless of Ignore.','要求满足所有选中条件；打开任何界面都会停止准备新的瞄准朝向，不受忽略条件影响。','要求滿足所有選取條件；開啟任何介面都會停止準備新的瞄準朝向，不受忽略條件影響。')
text('requires.emptyHand','Require the actual main-hand item stack to be empty.','要求实际主手物品堆为空。','要求實際主手物品堆為空。')
text('angleSmooth','Choose Interpolation, Sigmoid or Linear smoothing before render interpolation.','选择插值、S 形或直线转向平滑，再应用逐帧插值。','選擇插值、S 形或直線轉向平滑，再套用逐影格插值。')
text('angleSmooth.interpolation','Combine Bezier and sigmoid factors with the previous RotationManager target direction change.','结合贝塞尔与 S 形速度系数，并根据转向管理器的上一次目标方向变化进行调整。','結合貝茲與 S 形速度係數，並根據轉向管理器的上一次目標方向變化進行調整。')
text('angleSmooth.interpolation.horizontalSpeed','Random horizontal percentage for each new smooth rotation.','每次计算平滑朝向时随机抽取的水平速度百分比。','每次計算平滑朝向時隨機抽取的水平速度百分比。')
text('angleSmooth.interpolation.verticalSpeed','Random vertical percentage for each new smooth rotation.','每次计算平滑朝向时随机抽取的垂直速度百分比。','每次計算平滑朝向時隨機抽取的垂直速度百分比。')
text('angleSmooth.interpolation.directionChangeFactor','Random extra factor for changes from the previous managed rotation target.','目标相对上一次管理的转向目标发生方向变化时，随机抽取的额外系数。','目標相對上一次管理的轉向目標發生方向變化時，隨機抽取的額外係數。')
text('angleSmooth.interpolation.midpoint','Normalized angle at which smoothing switches between sigmoid and Bezier factors.','按归一化转向角度，在 S 形与贝塞尔系数之间切换的阈值。','按正規化轉向角度，在 S 形與貝茲係數之間切換的門檻。')
text('angleSmooth.sigmoid','Limit component turn speeds using a sigmoid curve based on the remaining angle.','根据剩余角度，使用 S 形曲线限制各轴的转向速度。','根據剩餘角度，使用 S 形曲線限制各軸的轉向速度。')
text('angleSmooth.linear','Limit horizontal and vertical rotation components by the sampled turn speeds.','按随机抽取的转向速度限制水平与垂直转动。','按隨機抽取的轉向速度限制水平與垂直轉動。')
text('axis','Apply the selected axes; unselected axes retain the stored tick rotation.','应用选中的瞄准轴；未选中的轴保留该游戏刻储存的玩家朝向。','套用選取的瞄準軸；未選取的軸保留該遊戲刻儲存的玩家朝向。')
text('axis.horizontal','Apply interpolated horizontal yaw.','应用插值后的水平朝向。','套用插值後的水平朝向。')
text('axis.vertical','Apply interpolated vertical pitch.','应用插值后的垂直俯仰角。','套用插值後的垂直俯仰角。')
text('ignore','Allow the stored aim to continue rendering in selected open-screen conditions.','允许储存的瞄准朝向在选中的界面状态下继续应用。','允許儲存的瞄準朝向在選取的介面狀態下繼續套用。')
text('ignore.screen','Continue rendering the stored aim while any screen is open.','打开界面时仍继续应用储存的瞄准朝向。','開啟介面時仍繼續套用儲存的瞄準朝向。')
text('ignore.container','Continue rendering the stored aim while a container screen or server inventory is open.','容器界面或服务器侧物品栏打开时，仍继续应用储存的瞄准朝向。','容器介面或伺服器端物品欄開啟時，仍繼續套用儲存的瞄準朝向。')
text('aimPoint','Choose a point on the current zero-tick target box, then apply persistent Delay, Lazy and Gaussian processors.','在当前目标碰撞箱上选择瞄准点，再依次应用持续保存状态的延迟、惰性与高斯处理。','在目前目標碰撞箱上選擇瞄準點，再依序套用持續儲存狀態的延遲、惰性與高斯處理。')
def update(folder):
 rod=runpy.run_path(str(HERE/'update-auto-rod-locales.py'))
 bow=runpy.run_path(str(HERE/'update-auto-bow-locales.py'))
 rendering=runpy.run_path(str(HERE/'update-target-renderer-locales.py'))['update']
 texts={k:v for k,v in rod['TEXTS'].items()if k.startswith('aimPoint.')or k.startswith('requires.')}
 for k,v in bow['TEXTS'].items():
  if k.startswith('bowAimbot.target.'):texts[k.removeprefix('bowAimbot.')]=v
 texts.update(TEXTS)
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['Aimbot'][index-1]};entries.update({k:v[index]for k,v in texts.items()})
  put_module(data,PREFIX,'Aimbot',entries,{k:v[0]for k,v in texts.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for field in ['descriptions','englishDescriptions']:
   old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations.angleSmooth.'
   for k,v in list(data.get(field,{}).items()):
    if k.startswith(old+'linear.')or k.startswith(old+'sigmoid.'):
     data[field][k.replace(old,PREFIX+'.angleSmooth.')]=v
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');rendering(locale,path,PREFIX+'.targetRendering')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');a=p.parse_args();update(a.folder)
