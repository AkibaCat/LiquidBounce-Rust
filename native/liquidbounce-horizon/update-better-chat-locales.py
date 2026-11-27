#!/usr/bin/env python3
"""Complete BetterChat and shared translation-provider Simplified/Traditional display text."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
'description':('Quality of life improvements to the in-game chat.','改进游戏内聊天的实用功能。','改善遊戲內聊天的實用功能。'),
'features.description':('Choose unlimited history, protection against clearing, chat after death and full-width outgoing text.','选择无限聊天记录、防止清屏、死亡后聊天与发送全角文本。','選擇無限聊天記錄、防止清除、死亡後聊天與傳送全形文字。'),
'appendPrefix.description':('Add the configured prefix to outgoing chat messages. Commands are unaffected.','在发出的聊天消息前添加指定前缀。','在傳出的聊天訊息前加入指定前綴。'),
'appendPrefix.enabled.description':('Enable outgoing message prefixes.','启用聊天消息前缀。','啟用聊天訊息前綴。'),
'appendPrefix.prefix.description':('Text inserted before every outgoing chat message.','添加到每条发出消息前面的文本。','加入至每則傳出訊息前方的文字。'),
'appendSuffix.description':('Add the configured suffix to outgoing chat messages.','在发出的聊天消息后添加指定后缀。','在傳出的聊天訊息後加入指定後綴。'),
'appendSuffix.enabled.description':('Enable outgoing message suffixes.','启用聊天消息后缀。','啟用聊天訊息後綴。'),
'appendSuffix.suffix.description':('Text appended to every outgoing chat message.','添加到每条发出消息后面的文本。','加入至每則傳出訊息後方的文字。'),
'autoTranslate.description':('Translate selected incoming message types using the shared AutoTranslate settings.','使用全局自动翻译设置翻译所选类型的接收消息。','使用全域自動翻譯設定翻譯所選類型的接收訊息。'),
'antiSpam.description':('Filter messages with regular expressions and optionally stack duplicates.','使用正则表达式过滤消息，并可合并重复消息。','使用正規表示式篩選訊息，並可合併重複訊息。'),
'antiSpam.enabled.description':('Enable incoming message filtering and duplicate stacking.','启用接收消息过滤与重复堆叠。','啟用接收訊息篩選與重複堆疊。'),
'antiSpam.stackMessages.description':('Replace earlier copies of the same message with one message and a repeat count.','将之前相同的消息替换为一条，并显示重复次数。','將先前相同的訊息取代為一則，並顯示重複次數。'),
'antiSpam.filters.description':('Java regular expressions that match the complete message body after the first greater-than sign.','匹配第一个大于号后完整正文的 Java 正则表达式。','比對第一個大於號後完整內文的 Java 正規表示式。'),
'copy.description':('Shift-left-click a message to copy it; right-click it to put the whole message in chat.','Shift 加左键复制消息；右键将整条消息放入聊天输入框。','Shift 加左鍵複製訊息；右鍵將整則訊息放入聊天輸入框。'),
'copy.enabled.description':('Enable whole-message copying across wrapped lines.','启用跨换行的整条消息复制。','啟用跨折行的整則訊息複製。'),
'copy.notify.description':('Show a success notification after copying a chat message.','复制聊天消息后显示成功通知。','複製聊天訊息後顯示成功通知。'),
'copy.highlight.description':('Highlight every visible wrapped line of the message under the mouse.','高亮鼠标下消息的所有可见换行。','醒目提示滑鼠下訊息的所有可見折行。'),
}
GLOBAL={
'description':('Shared incoming-chat translation provider and target language.','接收聊天消息翻译共用的提供方与目标语言。','接收聊天訊息翻譯共用的提供者與目標語言。'),
'languageCode.description':('Target translation language code; defaults to en independently of the game language.','翻译目标语言代码，默认 en，与游戏显示语言独立。','翻譯目標語言代碼，預設 en，與遊戲顯示語言獨立。'),
'provider.description':('Select Google Translate or a configurable LibreTranslate instance.','选择 Google 翻译或可配置的 LibreTranslate 实例。','選擇 Google 翻譯或可設定的 LibreTranslate 執行個體。'),
'provider.libreTranslate.baseUrl.description':('LibreTranslate instance base URL without the /translate path.','LibreTranslate 实例的基础地址，不包含 /translate 路径。','LibreTranslate 執行個體的基底位址，不含 /translate 路徑。'),
'provider.libreTranslate.apiKey.description':('API key for the LibreTranslate instance; leave empty when none is required.','LibreTranslate 实例的 API 密钥，不需要时留空。','LibreTranslate 執行個體的 API 金鑰，不需要時留空。'),
}
LABELS={
'Features':('功能','功能'),'Infinite':('无限记录','無限記錄'),'AntiClear':('防止清屏','防止清除'),'KeepAfterDeath':('死亡后聊天','死亡後聊天'),'ForceUnicodeChat':('发送全角文本','傳送全形文字'),
'AppendPrefix':('添加前缀','加入前綴'),'AppendSuffix':('添加后缀','加入後綴'),'Prefix':('前缀','前綴'),'Suffix':('后缀','後綴'),'AutoTranslate':('自动翻译','自動翻譯'),'AntiSpam':('消息防刷屏','訊息防洗版'),'StackMessages':('堆叠重复消息','堆疊重複訊息'),'Filters':('过滤表达式','篩選表示式'),'Copy':('复制消息','複製訊息'),'Notify':('复制通知','複製通知'),'Highlight':('悬停高亮','游標醒目提示'),'ChatMessage':('玩家消息','玩家訊息'),'DisguisedChatMessage':('伪装聊天消息','偽裝聊天訊息'),'GameMessage':('游戏消息','遊戲訊息'),'LanguageCode':('目标语言代码','目標語言代碼'),'Provider':('翻译提供方','翻譯提供者'),'Google':('Google 翻译','Google 翻譯'),'LibreTranslate':('LibreTranslate','LibreTranslate'),'BaseUrl':('服务地址','服務位址'),'ApiKey':('API 密钥','API 金鑰'),'ChatCopy':('聊天复制','聊天複製'),'The line is copied':('已复制这条消息','已複製這則訊息')}
def update(folder):
 for code,index in [('zh_cn',1),('zh_tw',2)]:
  path=folder/f'display_{code}.json';data=json.loads(path.read_text(encoding='utf8'))
  put_module(data,'liquidbounce.module.betterChat','BetterChat',{'':('聊天增强','聊天增強')[index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
  for k,v in GLOBAL.items():
   key='liquidbounce.autoTranslate.'+k;data.setdefault('descriptions',{})[key]=v[index]
   if 'englishDescriptions'in data:data['englishDescriptions'][key]=v[0]
  for k,v in LABELS.items():data.setdefault('labels',{})[k]=v[index-1]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(ap.parse_args().folder)
