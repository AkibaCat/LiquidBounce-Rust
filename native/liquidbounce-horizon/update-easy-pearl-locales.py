#!/usr/bin/env python3
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
LABELS={'EasyPearl':('珍珠辅助','珍珠輔助'),'PearlHelper':('珍珠助手','珍珠助手'),'PearlAssist':('珍珠瞄准辅助','珍珠瞄準輔助'),'PearlTP':('珍珠传送','珍珠傳送'),'AimOffThreshold':('瞄准允许偏差','瞄準允許偏差'),'ReachableCheck':('可达性检查','可達性檢查'),'ShowDistance':('显示距离','顯示距離')}
TEXTS={
 'description':('Throw an ender pearl at the position you are looking at.','自动调整末影珍珠的投掷角度，使其落在所瞄准的位置。','自動調整終界珍珠的投擲角度，使其落在所瞄準的位置。'),
 'aimOffThreshold.description':('Throw once the server view differs from the solved direction by at most this angle.','服务器视线与计算出的投掷方向之间的夹角不超过此值时投掷。','伺服器視線與計算出的投擲方向之間的夾角不超過此值時投擲。'),
 'reachableCheck.description':('Cancel a throw at an unreachable block and display the original warning.','瞄准无法到达的方块时取消投掷，并显示原版警告。','瞄準無法到達的方塊時取消投擲，並顯示原版警告。'),
 'showDistance.description':('Show the distance from your feet to the aimed position in the world.','在世界中显示玩家脚部到瞄准位置的距离。','在世界中顯示玩家腳部到瞄準位置的距離。')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  entries={'':LABELS['EasyPearl'][index-1]};entries.update({k:v[index]for k,v in TEXTS.items()})
  put_module(data,'liquidbounce.module.easyPearl','EasyPearl',entries,{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  old='liquidbounce.module.noRotateSet.mode.resetRotation.rotations'
  for field in ['descriptions','englishDescriptions']:
   for key,value in list(data.get(field,{}).items()):
    if key.startswith(old):data[field][key.replace(old,'liquidbounce.module.easyPearl.rotations')]=value
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');args=a.parse_args();update(args.folder)
