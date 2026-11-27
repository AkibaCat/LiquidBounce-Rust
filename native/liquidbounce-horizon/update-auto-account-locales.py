#!/usr/bin/env python3
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
HELP={
'en_us':{'password':'Password used for login and registration. Stored with private client settings.','delay':'Random delay in game ticks after a connection is available.','registerCommand':'Registration command without its leading slash.','loginCommand':'Login command without its leading slash.','registerRegex':'Java regular expression matching registration prompts. Checked before the login expression.','loginRegex':'Java regular expression matching login prompts.','repeatPasswordOnRegister':'Include the password twice in registration commands.','messageSource':'Watch chat, title and subtitle messages for account prompts.'},
'zh_cn':{'password':'用于登录和注册的密码，随客户端私有配置保存。','delay':'连接可用后随机等待的游戏刻数。','registerCommand':'注册命令，不包含开头的斜杠。','loginCommand':'登录命令，不包含开头的斜杠。','registerRegex':'匹配注册提示的 Java 正则表达式，优先于登录规则检查。','loginRegex':'匹配登录提示的 Java 正则表达式。','repeatPasswordOnRegister':'注册命令中重复输入两次密码。','messageSource':'从聊天、标题和副标题中识别登录与注册提示。'},
'zh_tw':{'password':'用於登入與註冊的密碼，隨客戶端私有設定儲存。','delay':'連線可用後隨機等待的遊戲刻數。','registerCommand':'註冊指令，不包含開頭的斜線。','loginCommand':'登入指令，不包含開頭的斜線。','registerRegex':'比對註冊提示的 Java 正規表示式，優先於登入規則檢查。','loginRegex':'比對登入提示的 Java 正規表示式。','repeatPasswordOnRegister':'註冊指令中重複輸入兩次密碼。','messageSource':'從聊天、標題與副標題中識別登入與註冊提示。'},
}
LABELS={'zh_cn':{'RegisterCommand':'注册命令','LoginCommand':'登录命令','RegisterRegex':'注册匹配规则','LoginRegex':'登录匹配规则','RepeatPasswordOnRegister':'重复注册密码','MessageSource':'消息来源','Subtitle':'副标题'},'zh_tw':{'RegisterCommand':'註冊指令','LoginCommand':'登入指令','RegisterRegex':'註冊比對規則','LoginRegex':'登入比對規則','RepeatPasswordOnRegister':'重複註冊密碼','MessageSource':'訊息來源','Subtitle':'副標題'}}
for language in ('zh_cn','zh_tw'):
 p=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json';d=json.loads(p.read_text(encoding='utf8'))
 original=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
 key='liquidbounce.module.autoAccount.description';d['descriptions'][key]=original[key]
 d['descriptions'].update({f'liquidbounce.module.autoAccount.{k}.description':v for k,v in HELP[language].items()})
 d['labels'].update(LABELS[language])
 if language=='zh_cn':
  original=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
  d['englishDescriptions'][key]=original[key]
  d['englishDescriptions'].update({f'liquidbounce.module.autoAccount.{k}.description':v for k,v in HELP['en_us'].items()})
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Merged AutoAccount Simplified/Traditional Chinese')
