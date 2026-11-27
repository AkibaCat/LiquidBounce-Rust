from pathlib import Path
import argparse,json
HERE=Path(__file__).resolve().parent
DESCRIPTIONS={
'protocol':('AI provider wire format.','AI 服务的通信协议。','AI 服務的通訊協定。'),
'baseUrl':('Provider API root; blank uses the selected protocol default.','服务的 API 根地址；留空使用所选协议的默认地址。','服務的 API 根位址；留空使用所選協定的預設位址。'),
'apiKey':('Provider API key. This module is excluded from public AutoConfig exports.','服务的 API 密钥；公开自动配置不包含此模块。','服務的 API 金鑰；公開自動設定不包含此模組。'),
'model':('Model identifier sent to the configured provider.','向所配置服务发送的模型标识。','向所設定服務傳送的模型識別碼。'),
'reactionTime':('Random delay before sending an answer or automatic gg, in milliseconds.','发送答案或自动 gg 前的随机等待时间，单位为毫秒。','傳送答案或自動 gg 前的隨機等待時間，單位為毫秒。'),
'cooldown':('Minutes to wait before accepting another chat game trigger.','接受下一次聊天游戏触发前等待的分钟数。','接受下一次聊天遊戲觸發前等待的分鐘數。'),
'bufferTime':('Collect system messages for this many milliseconds after the trigger.','触发后收集系统消息的时间，单位为毫秒。','觸發後收集系統訊息的時間，單位為毫秒。'),
'triggerSentence':('Case-sensitive text that begins a chat game question.','开始收集聊天游戏题目的文字，区分大小写。','開始收集聊天遊戲題目的文字，區分大小寫。'),
'includeTrigger':('Include the trigger message in the question sent to the AI provider.','把包含触发词的消息一同发送给 AI 服务。','把包含觸發詞的訊息一同傳送給 AI 服務。'),
'serverName':('Replace {SERVER_NAME} in the prompt with this server name.','将提示词中的 {SERVER_NAME} 替换为此服务器名称。','將提示詞中的 {SERVER_NAME} 替換為此伺服器名稱。'),
'answerTemplate':('Java format string applied when sending; %s is the answer. A leading slash sends a command.','发送时使用的 Java 格式字符串；%s 表示答案。以斜杠开头时发送命令。','傳送時使用的 Java 格式字串；%s 表示答案。以斜線開頭時傳送指令。'),
'prompt':('System prompt sent with each question.','每次提问时发送的系统提示词。','每次提問時傳送的系統提示詞。'),
}
def update(directory):
 for code,i in [('zh_cn',1),('zh_tw',2)]:
  p=directory/f'display_{code}.json';d=json.loads(p.read_text(encoding='utf8'));d['moduleNames']['AutoChatGame']=('自动聊天答题','自動聊天答題')[i-1]
  labels={'Protocol':('通信协议','通訊協定'),'BaseUrl':('API 根地址','API 根位址'),'ApiKey':('API 密钥','API 金鑰'),'Model':('模型','模型'),'ReactionTime':('反应时间','反應時間'),'BufferTime':('缓冲时间','緩衝時間'),'TriggerSentence':('触发文字','觸發文字'),'IncludeTrigger':('包含触发消息','包含觸發訊息'),'ServerName':('服务器名称','伺服器名稱'),'AnswerTemplate':('答案模板','答案範本'),'Prompt':('提示词','提示詞'),'OpenAiCompatible':('OpenAI 兼容','OpenAI 相容')}
  for label,texts in labels.items():d['labels'][label]=texts[i-1]
  for key,texts in DESCRIPTIONS.items():
   key='liquidbounce.module.autoChatGame.'+key+'.description';d['descriptions'][key]=texts[i]
   if i==1:d['englishDescriptions'][key]=texts[0]
  p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(code,len(DESCRIPTIONS),'AutoChatGame descriptions')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(a.parse_args().directory)
