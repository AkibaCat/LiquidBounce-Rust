from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2];FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'PacketLogger':('数据包记录','封包記錄'),'C2SPackets':('客户端发送的数据包','用戶端傳送的封包'),'S2CPackets':('服务器发送的数据包','伺服器傳送的封包'),'ShowFieldType':('显示字段类型','顯示欄位型別'),'OutputTarget':('输出目标','輸出目標'),'Chat':('聊天','聊天'),'File':('文件','檔案')}
TEXTS={
'':('Log actual incoming and outgoing packets, including fields and cancellation.','记录实际收发的数据包、字段和取消状态。','記錄實際收發的封包、欄位與取消狀態。'),
'filter':('Choose whether the packet lists allow or exclude their entries.','选择列表作为允许记录的白名单，或排除记录的黑名单。','選擇清單作為允許記錄的白名單，或排除記錄的黑名單。'),
'c2SPackets':('Outgoing packet identifiers used by the filter.','过滤器使用的客户端发出数据包标识符。','篩選器使用的用戶端發出封包識別碼。'),
's2CPackets':('Incoming packet identifiers used by the filter.','过滤器使用的服务器发来数据包标识符。','篩選器使用的伺服器傳入封包識別碼。'),
'showFieldType':('Include each field type in chat. Click a field, type or value to copy it.','在聊天中显示字段类型；点击字段、类型或值可复制。','在聊天中顯示欄位型別；點擊欄位、型別或值可複製。'),
'outputTarget':('Send logs to chat, CSV files in packet-logger, or both.','将日志输出到聊天、packet-logger 目录中的 CSV 文件，或同时输出。','將記錄輸出至聊天、packet-logger 目錄中的 CSV 檔案，或同時輸出。')}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
 p=FOLDER/f'display_{locale}.json';d=json.loads(p.read_text(encoding='utf8'));d['moduleNames']['PacketLogger']=LABELS['PacketLogger'][index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
 for suffix,text in TEXTS.items():
  key='liquidbounce.module.packetLogger'+('.'+suffix if suffix else '')+'.description';d['descriptions'][key]=text[index]
  if locale=='zh_cn':d['englishDescriptions'][key]=text[0]
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'PacketLogger help entries')
