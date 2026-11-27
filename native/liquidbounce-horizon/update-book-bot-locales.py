from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
'mode.random.pages':('Number of pages generated in Random mode.','随机模式生成的页数。','隨機模式產生的頁數。'),
'mode.random.asciiOnly':('Generate only printable ASCII characters.','仅生成可打印的 ASCII 字符。','僅產生可列印的 ASCII 字元。'),
'mode.random.allowSpace':('Allow Unicode whitespace in the random character stream.','允许随机字符流包含 Unicode 空白字符。','允許隨機字元流包含 Unicode 空白字元。'),
'mode.file.pages':('Maximum pages generated from the selected text file.','从所选文本文件生成的最大页数。','從所選文字檔產生的最大頁數。'),
'mode.file.cyclic':('Read the text file again from the beginning after reaching its end.','读取到文件末尾后从开头循环读取。','讀取到檔案結尾後從開頭循環讀取。'),
'mode.file.source':('Text file used to fill books.','用于填充书本的文本文件。','用於填充書本的文字檔。'),
'sign':('Sign the book after filling its pages.','填充书页后为书本签名。','填充書頁後為書本簽名。'),
'sign.name':('Book title; %count% is replaced by the number of books generated since enabling.','书名；%count% 会替换为本次启用后已生成的书本数量。','書名；%count% 會替換為本次啟用後已產生的書本數量。'),
'delay':('Minimum time between writing books.','两次写书之间的最短等待时间。','兩次寫書之間的最短等待時間。')}
for lang,index in [('zh_cn',1),('zh_tw',2)]:
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json';d=json.loads(p.read_text(encoding='utf8'))
 for category in ['descriptions','englishDescriptions']:
  if category not in d:continue
  d[category].update({key.replace('elytraSwap','bookBot'):value for key,value in list(d[category].items()) if key.startswith('liquidbounce.module.elytraSwap.constraints.')})
 d['descriptions'].update({f'liquidbounce.module.bookBot.{k}.description':v[index] for k,v in HELP.items()})
 d['labels'].update({'Pages':'页数' if index==1 else '頁數','AsciiOnly':'仅 ASCII' if index==1 else '僅 ASCII','AllowSpace':'允许空白字符' if index==1 else '允許空白字元','Cyclic':'循环读取' if index==1 else '循環讀取','Sign':'签名' if index==1 else '簽名'})
 if index==1:d['englishDescriptions'].update({f'liquidbounce.module.bookBot.{k}.description':v[0] for k,v in HELP.items()})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged BookBot simplified/traditional Chinese')
