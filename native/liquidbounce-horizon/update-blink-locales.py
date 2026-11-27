#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'Directions':('数据包方向','封包方向'),'Incoming':('接收','接收'),'Outgoing':('发送','傳送'),'Dummy':('显示假人','顯示假人'),'Ambush':('伏击释放','伏擊釋放'),'AutoReset':('自动重置','自動重設'),'ResetAfter':('重置间隔','重設間隔'),'ResetAction':('重置方式','重設方式')}
TEXTS={
 '':('Queue selected incoming or outgoing packets and release them later.','暂存所选方向的数据包，并在关闭或满足条件时释放。','暫存所選方向的封包，並在關閉或符合條件時釋放。'),
 'directions':('Packet directions that Blink queues.','选择暂存接收、发送或两个方向的数据包。','選擇暫存接收、傳送或兩個方向的封包。'),
 'dummy':('Create a local player clone at the position where Blink was enabled.','启用时在当前位置创建本地假人。','啟用時在目前位置建立本機假人。'),
 'ambush':('Disable Blink and release packets before an entity interaction or attack.','与实体交互或攻击前关闭模块，释放已暂存的数据包。','與實體互動或攻擊前關閉模組，釋放已暫存的封包。'),
 'autoDisable':('Disable Blink after an automatic reset.','自动重置后关闭模块。','自動重設後關閉模組。'),
 'autoReset.enabled':('Periodically reset or release the queued packets.','定期回到暂存起点，或释放已暂存的数据包。','定期回到暫存起點，或釋放已暫存的封包。'),
 'autoReset.resetAfter':('Reset after more than this many player movement ticks.','玩家移动刻数超过此值后触发重置。','玩家移動刻數超過此值後觸發重設。'),
 'autoReset.resetAction':('Reset discards queued movement and returns to its first position; Blink sends the selected directions.','重置会丢弃移动包并返回首个暂存位置；释放会发送所选方向的数据包。','重設會捨棄移動封包並返回首個暫存位置；釋放會傳送所選方向的封包。'),
}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 path=FOLDER/f'display_{locale}.json';d=json.loads(path.read_text(encoding='utf8'));d['moduleNames']['Blink']=('数据包暂存','封包暫存')[index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
 for suffix,v in TEXTS.items():
  key='liquidbounce.module.blink'+('.'+suffix if suffix else '')+'.description';d['descriptions'][key]=v[index]
  if locale=='zh_cn':d['englishDescriptions'][key]=v[0]
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'Blink help entries')
