"""FakeLag Simplified/Traditional descriptions are selected by the live game locale."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
 'description':('Hold outgoing packets to avoid enemy hits, sharing the global Blink queue.','通过暂缓发送数据包躲避敌人攻击，与其他模块共享数据包队列。','透過暫緩傳送封包躲避敵人攻擊，與其他模組共用封包佇列。'),
 'range.description':('Minimum and maximum target hitbox distance for nearby enemies; the maximum also bounds dynamic checks at the queued server position.','检测附近敌人时的碰撞箱最小与最大距离；动态模式也用最大距离检查队列中的服务器位置。','偵測附近敵人時的碰撞箱最小與最大距離；動態模式也以最大距離檢查佇列中的伺服器位置。'),
 'delay.description':('Random hold limit in milliseconds, sampled initially and again when the oldest queued packet reaches the limit.','数据包暂缓时间上限，单位毫秒；初始化及队列最旧数据包达到上限时重新随机。','封包暫緩時間上限，單位毫秒；初始化及佇列最舊封包達到上限時重新隨機。'),
 'recoilTime.description':('Milliseconds after a selected flush interaction before queuing is allowed again.','发生指定的立即发送交互后，再次允许暂缓数据包前等待的毫秒数。','發生指定的立即傳送互動後，再次允許暫緩封包前等待的毫秒數。'),
 'mode.description':('Constant always queues eligible packets; Dynamic keeps lag only with nearby enemies when the queued position is at least as safe as the client position.','恒定模式暂缓所有符合条件的数据包；动态模式仅在附近有敌人，且队列位置不比客户端位置更危险时暂缓。','恆定模式暫緩所有符合條件的封包；動態模式僅在附近有敵人，且佇列位置不比用戶端位置更危險時暫緩。'),
 'flushOn.description':('Interactions that stop FakeLag queuing and reset its recoil timer. Other modules can still keep the shared queue held.','停止假延迟暂缓并重置恢复计时器的交互；其他模块仍可能继续保持共享队列。','停止假延遲暫緩並重設恢復計時器的互動；其他模組仍可能繼續保持共用佇列。'),
}
LABELS={'FakeLag':('假延迟','假延遲'),'RecoilTime':('恢复等待时间','恢復等待時間'),'FlushOn':('立即发送条件','立即傳送條件'),'EntityInteract':('实体交互','實體互動'),'BlockInteract':('方块交互','方塊互動')}
def update(folder):
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
  put_module(data,'liquidbounce.module.fakeLag','FakeLag',{'':LABELS['FakeLag'][index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
  data['labels'].update({k:v[index-1]for k,v in LABELS.items()});path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(p.parse_args().folder)
