#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={'en_us':'Delay incoming KeepAlive and Ping packets by this many milliseconds. Disabling flushes the queue; disconnecting discards packets belonging to the old connection.','zh_cn':'将收到的保活和延迟检测包延后指定毫秒数处理。关闭模块会释放队列；断开连接会丢弃旧连接的数据包。','zh_tw':'將收到的保活與延遲偵測封包延後指定毫秒數處理。關閉模組會釋放佇列；中斷連線會捨棄舊連線的封包。'}
for lang in ('zh_cn','zh_tw'):
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'));key='liquidbounce.module.pingSpoof.delay.description';d['descriptions'][key]=HELP[lang]
 if lang=='zh_cn':d['englishDescriptions'][key]=HELP['en_us']
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged PingSpoof simplified/traditional Chinese')
