#!/usr/bin/env python3
"""ReportHelper complete original configuration in Simplified/Traditional Chinese."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'ReportHelper':('举报助手','檢舉助手'),'AutoReport':('自动举报','自動檢舉'),'AutoConfirm':('自动确认举报','自動確認檢舉'),'CommandPattern':('举报命令格式','檢舉指令格式'),'Hypixel':('Hypixel','Hypixel'),'Heypixel':('Heypixel','Heypixel')}
TEXTS={
'':('Reports other players named with you in chat and confirms supported report menus.','举报聊天中与你同时出现的玩家，并自动确认受支持服务器的举报菜单。','檢舉聊天中與你同時出現的玩家，並自動確認支援伺服器的檢舉選單。'),
'autoReport.enabled':('Automatically report a non-friend whose exact profile name appears with yours in chat.','聊天同时包含你的名字与非好友的准确玩家名时，自动举报该玩家。','聊天同時包含你的名字與非好友的精確玩家名稱時，自動檢舉該玩家。'),
'autoReport.delay':('Random tick delay before sending the report command.','发送举报命令前随机等待的游戏刻数。','傳送檢舉指令前隨機等待的遊戲刻數。'),
'autoReport.chance':('Probability of reporting each eligible chat occurrence.','每次符合条件的聊天触发举报的概率。','每次符合條件的聊天觸發檢舉的機率。'),
'autoReport.commandPattern':('Java format string for the command; %s inserts the player name.','举报命令的 Java 格式字符串；%s 代表被举报的玩家名。','檢舉指令的 Java 格式字串；%s 代表被檢舉的玩家名稱。'),
'autoConfirm.enabled':('Confirm a recognized report menu and close it locally.','识别举报菜单后点击确认，并关闭客户端界面。','辨識檢舉選單後點選確認，並關閉用戶端介面。'),
'autoConfirm.mode':('Choose the server report menu layout.','选择服务器举报菜单的布局。','選擇伺服器檢舉選單的版面配置。'),
}
def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['ReportHelper']=LABELS['ReportHelper'][index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
        for suffix,values in TEXTS.items():
            key='liquidbounce.module.reportHelper'+('.'+suffix if suffix else '')+'.description';data['descriptions'][key]=values[index]
            if locale=='zh_cn':data['englishDescriptions'][key]=values[0]
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'ReportHelper descriptions')
if __name__=='__main__':main()
