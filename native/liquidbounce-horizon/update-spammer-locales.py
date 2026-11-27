#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
 'en_us':{
 'delay':'Time between message batches in seconds.','mps':'Random number of messages sent in each batch.',
 'messageSource.setting.message':'Messages selected randomly or in sequence.','messageSource.file.source':'UTF-8 text file, one message per line. Relative paths use the client config directory. File changes are detected automatically.',
 'pattern':'Choose random messages or cycle through them in order.','messageConverter':'Keep the text unchanged, apply leetspeak, randomize case, or insert random spaces.',
 'customFormatter':'Expand %f, %i and %s as random values and @a as another online player. When disabled, add a random prefix and randomize the message case.',
 },
 'zh_cn':{
 'delay':'每批消息之间的间隔，单位为秒。','mps':'每批随机发送的消息数量。',
 'messageSource.setting.message':'按随机或顺序方式选取的消息列表。','messageSource.file.source':'每行一条消息的 UTF-8 文本文件。相对路径以客户端配置目录为起点，文件变化后自动重新读取。',
 'pattern':'随机选取消息，或按列表顺序循环发送。','messageConverter':'保持原文、使用数字字母替换、随机大小写或随机插入空格。',
 'customFormatter':'将 %f、%i、%s 替换为随机值，将 @a 替换为其他在线玩家名。关闭时添加随机前缀并随机转换消息大小写。',
 },
 'zh_tw':{
 'delay':'每批訊息之間的間隔，單位為秒。','mps':'每批隨機傳送的訊息數量。',
 'messageSource.setting.message':'以隨機或順序方式選取的訊息清單。','messageSource.file.source':'每行一則訊息的 UTF-8 文字檔。相對路徑以用戶端設定目錄為起點，檔案變更後自動重新讀取。',
 'pattern':'隨機選取訊息，或依清單順序循環傳送。','messageConverter':'保留原文、使用數字字母替換、隨機大小寫或隨機插入空格。',
 'customFormatter':'將 %f、%i、%s 替換為隨機值，將 @a 替換為其他線上玩家名稱。關閉時加入隨機前綴並隨機轉換訊息大小寫。',
 }}
MESSAGES={'en_us':{'tooLong':'Spammer message is too long! (Max 256 characters)','failed':'Failed to get spammer message'},'zh_cn':{'tooLong':'刷屏消息过长！（最多 256 个字符）','failed':'获取刷屏消息失败'},'zh_tw':{'tooLong':'洗頻訊息過長！（最多 256 個字元）','failed':'取得洗頻訊息失敗'}}
for lang in ('zh_cn','zh_tw'):
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 d['descriptions'].update({f'liquidbounce.module.spammer.{k}.description':v for k,v in HELP[lang].items()})
 d['descriptions'].update({f'liquidbounce.module.spammer.messages.{k}':v for k,v in MESSAGES[lang].items()})
 d['labels'].update({'MessageSource':'消息来源' if lang=='zh_cn' else '訊息來源','MPS':'每批消息数' if lang=='zh_cn' else '每批訊息數','MessageConverter':'消息转换' if lang=='zh_cn' else '訊息轉換','CustomFormatter':'自定义格式' if lang=='zh_cn' else '自訂格式','Random Case':'随机大小写' if lang=='zh_cn' else '隨機大小寫','Random Space':'随机空格' if lang=='zh_cn' else '隨機空格','Leet':'数字字母替换' if lang=='zh_cn' else '數字字母替換'})
 if lang=='zh_cn':
  d['englishDescriptions'].update({f'liquidbounce.module.spammer.{k}.description':v for k,v in HELP['en_us'].items()})
  d['englishDescriptions'].update({f'liquidbounce.module.spammer.messages.{k}':v for k,v in MESSAGES['en_us'].items()})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged Spammer simplified/traditional Chinese')
