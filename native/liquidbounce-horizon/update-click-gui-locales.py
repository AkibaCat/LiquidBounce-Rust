"""Complete ModuleClickGui labels and descriptions in simplified/traditional Chinese."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Shows a menu to toggle and configure modules.','显示用于开关和配置模块的菜单。','顯示用於開關及設定模組的選單。'),
 'scale.description':('Scale of the ClickGUI.','模块菜单的显示缩放比例。','模組選單的顯示縮放比例。'),
 'searchBarAutoFocus.description':('Automatically focus the search bar when opening the menu.','打开菜单时自动聚焦搜索框。','開啟選單時自動聚焦搜尋框。'),
 'snapping.description':('Snap module panels to a grid.','将模块面板对齐到网格。','將模組面板對齊至網格。'),
 'snapping.enabled.description':('Enable panel snapping.','启用面板网格吸附。','啟用面板網格吸附。'),
 'snapping.gridSize.description':('Panel snapping grid size in pixels.','面板吸附网格的像素大小。','面板吸附網格的像素大小。'),
 'cache.description':('Keep a separate cached browser page for the ClickGUI.','为模块菜单保留独立的浏览器缓存页面。','為模組選單保留獨立的瀏覽器快取頁面。'),
 'renderer.description':('Browser rendering settings.','浏览器渲染设置。','瀏覽器繪製設定。'),
 'renderer.fps.description':('Browser frame rate; zero follows the current monitor refresh rate.','浏览器帧率；设为 0 时跟随当前显示器刷新率。','瀏覽器幀率；設為 0 時跟隨目前顯示器更新率。'),
 'renderer.syncGameFps.description':('Limit game frame rate to the browser rate while a client screen is open.','打开客户端界面时，将游戏帧率限制为浏览器帧率。','開啟用戶端介面時，將遊戲幀率限制為瀏覽器幀率。'),
}
LABELS={'SearchBarAutoFocus':('自动聚焦搜索框','自動聚焦搜尋框'),'Snapping':('网格吸附','網格吸附'),'GridSize':('网格大小','網格大小'),'Cache':('独立页面缓存','獨立頁面快取'),'Renderer':('渲染器','繪製器'),'Fps':('帧率','幀率'),'SyncGameFps':('同步游戏帧率','同步遊戲幀率')}
def update(folder):
 for code,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{code}.json';data=json.loads(path.read_text(encoding='utf8'))
  put_module(data,'liquidbounce.module.clickGui','ClickGUI',{'':('模块菜单','模組選單')[index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
  for k,v in LABELS.items():data.setdefault('labels',{})[k]=v[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(ap.parse_args().folder)
