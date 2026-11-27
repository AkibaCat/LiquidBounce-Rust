from pathlib import Path
import json,argparse
HERE=Path(__file__).resolve().parent
DESCRIPTIONS={
'behavior':('Screen policy: Normal, Safe (block inventory clicks while moving), Undetectable (stop in containers), or StopOnAction (pause movement before sending container actions).','界面行为：普通；安全（移动时拦截背包点击）；隐蔽（容器中停步）；操作时停步（先停步再发送容器操作）。','畫面行為：一般；安全（移動時攔截背包點擊）；隱蔽（容器中停步）；操作時停步（先停步再傳送容器操作）。'),
'passthroughSneak':('Read the physical sneak binding through supported screens. Text fields still suppress movement.','在允许的界面中读取实际潜行按键；编辑文字时仍停止移动。','在允許的畫面中讀取實際潛行按鍵；編輯文字時仍停止移動。'),
'sprintControl.enabled':('Override sprint while a container or the server-side player inventory is open.','容器或服务器端背包打开时覆盖冲刺状态。','容器或伺服器端背包開啟時覆寫衝刺狀態。'),
'sprintControl.client':('Keep, force, or suppress client sprint. Forcing sprint requires directional movement.','保持、强制或禁止客户端冲刺；强制冲刺要求正在移动。','保持、強制或禁止用戶端衝刺；強制衝刺要求正在移動。'),
'sprintControl.server':('Keep, force, or suppress server sprint. Applied after client control for the sprint key.','保持、强制或禁止服务器冲刺；冲刺按键以此设置为最终结果。','保持、強制或禁止伺服器衝刺；衝刺按鍵以此設定為最終結果。'),
'sneakControl.enabled':('Override sneak while a container or server-side player inventory is open.','容器或服务器端背包打开时覆盖潜行输入。','容器或伺服器端背包開啟時覆寫潛行輸入。'),
'sneakControl.client':('Keep, force, or suppress the client sneak input.','保持、强制或禁止客户端潜行输入。','保持、強制或禁止用戶端潛行輸入。'),
'timer.enabled':('Request the shared game timer while a container screen is open.','打开容器界面时向共享计时器请求速度。','開啟容器畫面時向共用計時器請求速度。'),
'timer.speed':('Game speed from 0.1 to 2.0, with InventoryMove timer priority.','游戏速度倍率（0.1 至 2.0），使用背包移动的计时器优先级。','遊戲速度倍率（0.1 至 2.0），使用背包移動的計時器優先順序。'),
'blink.enabled':('Pass container actions and queue other outgoing packets in the shared Blink manager while a container screen is open.','容器界面打开时放行容器操作，其余出站数据包进入共享 Blink 队列。','容器畫面開啟時放行容器操作，其餘傳出封包進入共用 Blink 佇列。'),
'blink.maximumTime':('Close the container after this many milliseconds and release the shared queue when other Blink users permit it.','达到此毫秒数后关闭容器；其他 Blink 使用者允许时释放共享队列。','達到此毫秒數後關閉容器；其他 Blink 使用者允許時釋放共用佇列。'),
}
def update(directory):
 for code,index in [('zh_cn',1),('zh_tw',2)]:
  p=directory/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'))
  d['moduleNames']['InventoryMove']=('背包移动','背包移動')[index-1]
  for label,texts in {'PassthroughSneak':('允许潜行按键','允許潛行按鍵'),'SprintControl':('冲刺控制','衝刺控制'),'SneakControl':('潜行控制','潛行控制'),'StopOnAction':('操作时停步','操作時停步'),'ForceSprint':('强制冲刺','強制衝刺'),'ForceNoSprint':('禁止冲刺','禁止衝刺'),'ForceSneak':('强制潜行','強制潛行'),'ForceNoSneak':('禁止潜行','禁止潛行'),'DoNotChange':('保持不变','保持不變')}.items():d['labels'][label]=texts[index-1]
  for key,texts in DESCRIPTIONS.items():
   key='liquidbounce.module.inventoryMove.'+key+'.description';d['descriptions'][key]=texts[index]
   if index==1:d['englishDescriptions'][key]=texts[0]
  p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(code,len(DESCRIPTIONS),'InventoryMove descriptions')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(a.parse_args().directory)
