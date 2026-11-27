"""Complete Debug descriptions, settings labels and runtime Chinese text."""
from pathlib import Path
import argparse, json
from display_locale_schema import put_module
HERE = Path(__file__).resolve().parent
TEXTS = {
 "description": ("Only of interest to developers.", "只有开发者会感兴趣。", "只有開發者會感興趣。"),
 "parameters.description": ("Display debug parameters grouped by owner.", "按所属对象分组显示调试参数。", "依所屬物件分組顯示偵錯參數。"),
 "global.description": ("Include parameters from owners outside modules, such as server and timer state.", "显示模块以外对象的参数，例如服务器和计时器状态。", "顯示模組以外物件的參數，例如伺服器與計時器狀態。"),
 "geometry.description": ("Display captured debug geometry in the world.", "在世界中显示捕获的调试几何图形。", "在世界中顯示擷取的偵錯幾何圖形。"),
 "expires.description": ("Seconds before an unrefreshed parameter expires.", "参数未刷新时经过多少秒后消失。", "參數未更新時經過多少秒後消失。"),
 "width.description": ("Width of each debug parameter table in pixels.", "每个调试参数表格的像素宽度。", "每個偵錯參數表格的像素寬度。"),
 "titleFilter.description": ("Regular expression that filters owner paths.", "用于筛选所属对象路径的正则表达式。", "用於篩選所屬物件路徑的正規表示式。"),
 "debugLog.description": ("Write parameter updates to a TSV file after enabling the module.", "启用模块后将参数更新写入 TSV 日志文件。", "啟用模組後將參數更新寫入 TSV 記錄檔。"),
 "simulatedPlayer.description": ("Draw the predicted path of the local player using actual movement inputs.", "根据实际移动输入绘制本地玩家的预测路径。", "根據實際移動輸入繪製本機玩家的預測路徑。"),
 "simulatedPlayer.enabled.description": ("Enable player movement prediction.", "启用玩家移动预测。", "啟用玩家移動預測。"),
 "simulatedPlayer.ticksToPredict.description": ("Number of movement ticks to simulate ahead.", "向前模拟移动的游戏刻数。", "向前模擬移動的遊戲刻數。"),
 "graph.description": ("Display the debug curve graph.", "显示调试曲线图。", "顯示偵錯曲線圖。"),
 "graph.enabled.description": ("Enable the debug curve graph.", "启用调试曲线图。", "啟用偵錯曲線圖。"),
 "graph.curve.description": ("Control points for the debug curve.", "调试曲线的控制点。", "偵錯曲線的控制點。"),
}
LABELS = {
 "Parameters": ("参数", "參數"), "Global": ("全局参数", "全域參數"), "Geometry": ("几何图形", "幾何圖形"),
 "Expires": ("过期时间", "到期時間"), "Width": ("宽度", "寬度"), "TitleFilter": ("标题筛选", "標題篩選"),
 "DebugLog": ("调试日志", "偵錯記錄"), "SimulatedPlayer": ("模拟玩家", "模擬玩家"), "TicksToPredict": ("预测刻数", "預測刻數"),
 "Graph": ("曲线图", "曲線圖"), "Curve": ("曲线", "曲線"), "X Axis": ("X 轴", "X 軸"), "Y Axis": ("Y 轴", "Y 軸"),
 "Client": ("客户端", "用戶端"), "Tick": ("游戏刻", "遊戲刻"), "Position": ("位置", "位置"), "Velocity": ("速度", "速度"),
 "Rotation": ("朝向", "朝向"), "OnGround": ("着地", "著地"), "Chunks": ("区块数", "區塊數"), "Entities": ("实体数", "實體數"), "Player": ("玩家", "玩家"),
 "rows above: {above}, below: {below}, scroll with chat open": ("上方 {above} 行，下方 {below} 行；打开聊天后滚动", "上方 {above} 列，下方 {below} 列；開啟聊天後捲動"),
}
def update(folder: Path):
 for code,index in [("zh_cn",1),("zh_tw",2)]:
  path=folder/f"display_{code}.json";data=json.loads(path.read_text(encoding="utf8"))
  put_module(data,"liquidbounce.module.debug","Debug",{"":("调试","偵錯")[index-1],**{key:value[index] for key,value in TEXTS.items()}},{key:value[0] for key,value in TEXTS.items()})
  for key,pair in LABELS.items():data.setdefault("labels",{})[key]=pair[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+"\n",encoding="utf8")
if __name__=="__main__":
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument("--folder",type=Path,default=HERE.parent/"liquidbounce-interop/resources/localization");update(ap.parse_args().folder)
