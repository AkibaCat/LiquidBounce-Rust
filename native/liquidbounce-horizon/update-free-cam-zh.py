#!/usr/bin/env python3
"""Complete FreeCam Simplified and Traditional display resources."""
from pathlib import Path
import argparse,json,sys
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'display_locale_schema.py').is_file()else HERE.parents[1]
sys.path.insert(0,str(NATIVE))
from display_locale_schema import put_module
PREFIX='liquidbounce.module.freeCam'
LABELS={
 'FreeCam':('自由相机','自由相機'),'Speed':('速度','速度'),
 'AllowCameraInteract':('允许相机交互','允許相機互動'),'LookAt':('身体看向目标','身體看向目標'),
 'CancelOn':('自动关闭条件','自動關閉條件'),'Damage':('受到伤害','受到傷害'),'Teleport':('收到位置修正','收到位置修正'),
 'Move':('身体水平移动','身體水平移動'),'Liquid':('身体进入液体','身體進入液體'),
 'Navigation':('导航身体','導航身體'),'Key':('导航按键','導航按鍵'),
 'MidClickCameraTeleport':('中键移动相机','中鍵移動相機'),'KeepSneaking':('保持潜行','保持潛行')}
TEXTS={
 'description':('Move the camera independently while the player body stays in place.','独立移动相机，同时保留玩家身体的位置。','獨立移動相機，同時保留玩家身體的位置。'),
 'speed.description':('Camera movement per game tick; horizontal diagonals keep the same speed.','每游戏刻相机移动的距离；水平斜向移动保持相同速度。','每遊戲刻相機移動的距離；水平斜向移動保持相同速度。'),
 'allowCameraInteract.description':('Pick and interact from the free camera position and view.','从自由相机的位置和视角选取并交互方块与实体。','從自由相機的位置與視角選取並互動方塊和實體。'),
 'allowCameraInteract.enabled.description':('Use the camera for interaction; when disabled use the body and its server rotation.','使用相机交互；关闭后使用身体位置和服务器旋转进行选取。','使用相機互動；關閉後使用身體位置和伺服器旋轉進行選取。'),
 'allowCameraInteract.lookAt.description':('Request a body rotation toward the current crosshair hit from the real player eyes.','让真实玩家的视线转向相机准星当前命中的位置。','讓真實玩家的視線轉向相機準星目前命中的位置。'),
 'cancelOn.description':('Disable FreeCam on the selected events; an empty selection never cancels automatically.','发生选中的事件时关闭自由相机；留空时不按这些条件自动关闭。','發生選取的事件時關閉自由相機；留空時不依這些條件自動關閉。'),
 'cancelOn.value.damage.description':('Disable when a server health update lowers the player health.','服务器生命值更新降低玩家生命值时关闭。','伺服器生命值更新降低玩家生命值時關閉。'),
 'cancelOn.value.teleport.description':('Disable on an incoming player position or teleport packet.','收到玩家位置修正或传送数据包时关闭。','收到玩家位置修正或傳送封包時關閉。'),
 'cancelOn.value.move.description':('Disable when the real player moves horizontally; gravity and vertical movement alone do not cancel.','真实玩家发生水平移动时关闭；单纯重力或垂直移动不会触发。','真實玩家發生水平移動時關閉；單純重力或垂直移動不會觸發。'),
 'cancelOn.value.liquid.description':('Disable when the player body touches water or lava.','玩家身体接触水或熔岩时关闭。','玩家身體接觸水或熔岩時關閉。'),
 'navigation.description':('Navigate the real player toward the camera outline ray, up to 200 blocks away.','让真实玩家沿路径前往相机轮廓射线指向的位置，最大射程 200 格。','讓真實玩家沿路徑前往相機輪廓射線指向的位置，最大射程 200 格。'),
 'navigation.enabled.description':('Enable the independent navigator; breaking and placing blocks are allowed by default.','启用独立导航器；默认允许破坏和放置方块。','啟用獨立導航器；預設允許破壞和放置方塊。'),
 'navigation.key.description':('Hold this key to move the body toward the camera target, stopping within 1.5 blocks. Default Left Control.','按住此键让身体前往相机目标，并在 1.5 格范围内停止；默认为左 Control。','按住此鍵讓身體前往相機目標，並在 1.5 格範圍內停止；預設為左 Control。'),
 'midClickCameraTeleport.description':('Move the camera 90 percent toward the 200-block outline hit when pressing the middle mouse button.','按下鼠标中键时，朝 200 格轮廓射线命中点移动相机，保留 10% 距离。','按下滑鼠中鍵時，朝 200 格輪廓射線命中點移動相機，保留 10% 距離。'),
 'keepSneaking.description':('Force the player body to sneak after other movement input handlers.','在其他移动输入处理之后，让玩家身体保持潜行。','在其他移動輸入處理之後，讓玩家身體保持潛行。')}
def update(folder,check=False):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['FreeCam'][index-1],**{k:v[index]for k,v in TEXTS.items()}}
  put_module(data,PREFIX,'FreeCam',entries,{k:v[0]for k,v in TEXTS.items()})
  scoped={'speed':'Speed','allowCameraInteract':'AllowCameraInteract','allowCameraInteract.lookAt':'LookAt','cancelOn':'CancelOn','navigation':'Navigation','navigation.key':'Key','midClickCameraTeleport':'MidClickCameraTeleport','keepSneaking':'KeepSneaking',**{'cancelOn.value.'+tag.lower():tag for tag in ['Damage','Teleport','Move','Liquid']}}
  for suffix,name in scoped.items():data['labels'][PREFIX+'.'+suffix]=LABELS[name][index-1]
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    old='liquidbounce.module.projectilePuncher.rotations'
    if key.startswith(old):data[field][key.replace(old,PREFIX+'.rotations',1)]=value
    old='liquidbounce.module.autoWalk.'
    if key.startswith(old)and not key[len(old):].startswith(('mode.','rotations.'))and key[len(old):]!='description':
     data[field][PREFIX+'.navigation.'+key[len(old):]]=value
  required=['angleSmooth.interpolation.horizontalSpeed','angleSmooth.aI.model','shortStop.rate','fail.factor']
  for suffix in required:
   if PREFIX+'.rotations.'+suffix+'.description'not in data['descriptions']:raise ValueError('Missing FreeCam rotation description: '+suffix)
  for key,value in list(data['labels'].items()):
   for old,new in [('liquidbounce.module.projectilePuncher.rotations',PREFIX+'.rotations'),('liquidbounce.module.autoWalk',PREFIX+'.navigation')]:
    if key.startswith(old+'.'):data['labels'][key.replace(old,new,1)]=value
  if not check:path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 print('FreeCam complete Simplified/Traditional resources '+('validated'if check else'updated'))
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--folder',type=Path,default=NATIVE.parent/'liquidbounce-interop/resources/localization');a.add_argument('--check',action='store_true');args=a.parse_args();update(args.folder,args.check)
