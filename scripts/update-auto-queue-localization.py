#!/usr/bin/env python3
"""Complete AutoQueue Simplified/Traditional Chinese labels and descriptions."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'AutoQueue':('自动重排','自動重排'),'AutoPlay':('自动重排','自動重排'),'Presets':('预设','預設'),'HypixelSW':('Hypixel 空岛战争','Hypixel 空島戰爭'),'GommeDuels':('Gomme 决斗','Gomme 決鬥'),'WinMessage':('胜利消息','勝利訊息'),'LoseMessage':('失败消息','失敗訊息'),'ControlKillAura':('控制杀戮光环','控制殺戮光環'),'WaitUntilWorldChange':('等待世界切换','等待世界切換'),'StartDelay':('开始延迟','開始延遲'),'MessageDelay':('消息间隔','訊息間隔'),'ChatTypes':('聊天类型','聊天類型'),'ChatMessage':('玩家聊天','玩家聊天'),'DisguisedChatMessage':('伪装聊天','偽裝聊天'),'GameMessage':('游戏消息','遊戲訊息'),'ClickGuiOpen':('点击界面打开时','點選介面開啟時'),'ChatScreenOpen':('聊天界面打开时','聊天介面開啟時'),'ContainerScreenOpen':('容器界面打开时','容器介面開啟時'),'PauseScreenOpen':('暂停菜单打开时','暫停選單開啟時'),'Keywords':('关键词','關鍵字'),'TabHeader':('玩家列表页眉','玩家清單頁首'),'TabFooter':('玩家列表页脚','玩家清單頁尾'),'UseItem':('使用物品','使用物品'),'EqualsIgnoreCase':('相等（忽略大小写）','相等（忽略大小寫）'),'ContainsIgnoreCase':('包含（忽略大小写）','包含（忽略大小寫）'),'StartsWithIgnoreCase':('开头匹配（忽略大小写）','開頭相符（忽略大小寫）'),'EndsWithIgnoreCase':('结尾匹配（忽略大小写）','結尾相符（忽略大小寫）')}
TEXTS={
'':('Automatically queue again using Hypixel SkyWars, Gomme Duels, or custom triggers and actions.','使用 Hypixel 空岛战争、Gomme 决斗预设，或自定义触发器与动作自动重新排队。','使用 Hypixel 空島戰爭、Gomme 決鬥預設，或自訂觸發器與動作自動重新排隊。'),
'pause':('Skip new queue attempts while any selected screen is open.','所选界面打开时，不开始新的重排流程。','所選介面開啟時，不開始新的重排流程。'),
'presets':('Select the server preset or a custom queue workflow.','选择服务器预设或自定义排队流程。','選擇伺服器預設或自訂排隊流程。'),
'presets.hypixelSW.gameMode':('Choose Normal or Insane solo SkyWars when paper is in your hotbar.','快捷栏有纸张时，加入普通或疯狂单人空岛战争。','快捷列有紙張時，加入普通或瘋狂單人空島戰爭。'),
'presets.gommeDuels.winMessage':('Chat message sent two real seconds after a recognized win.','识别到胜利后，等待两秒实际时间发送的聊天消息。','辨識到勝利後，等待兩秒實際時間傳送的聊天訊息。'),
'presets.gommeDuels.loseMessage':('Chat message sent two real seconds after a recognized loss.','识别到失败后，等待两秒实际时间发送的聊天消息。','辨識到失敗後，等待兩秒實際時間傳送的聊天訊息。'),
'presets.gommeDuels.controlKillAura':('Disable KillAura outside a match and enable it when the match starts.','对局外关闭杀戮光环，对局开始时开启。','對局外關閉殺戮光環，對局開始時開啟。'),
'presets.custom.trigger':('Choose which observation starts the custom workflow.','选择启动自定义流程的触发条件。','選擇啟動自訂流程的觸發條件。'),
'presets.custom.trigger.title.keywords':('Case-sensitive substrings in a received title. The pinned subtitle choice shares the Title name.','收到标题时按区分大小写的子串匹配。固定版本的副标题选项同样命名为 Title。','收到標題時依區分大小寫的子字串比對。固定版本的副標題選項同樣命名為 Title。'),
'presets.custom.trigger.message.keywords':('Case-sensitive substrings in a selected received chat type.','在所选类型的收到聊天中，按区分大小写的子串匹配。','在所選類型的收到聊天中，依區分大小寫的子字串比對。'),
'presets.custom.trigger.message.chatTypes':('Receive player chat, disguised chat, and/or game messages. At least one type is required.','选择玩家聊天、伪装聊天或游戏消息；至少选择一种。','選擇玩家聊天、偽裝聊天或遊戲訊息；至少選擇一種。'),
'presets.custom.trigger.tabHeader.text':('Case-sensitive substring required in the actual player-list header.','真实玩家列表页眉需要包含的区分大小写文本。','實際玩家清單頁首必須包含的區分大小寫文字。'),
'presets.custom.trigger.tabFooter.text':('Case-sensitive substring required in the actual player-list footer.','真实玩家列表页脚需要包含的区分大小写文本。','實際玩家清單頁尾必須包含的區分大小寫文字。'),
'presets.custom.action':('Send chat messages or select and use a matching item.','发送聊天消息，或选择并使用匹配的物品。','傳送聊天訊息，或選取並使用相符的物品。'),
'presets.custom.action.chat.startDelay':('Random real-time delay before the first message, in milliseconds.','发送第一条消息前随机等待的实际毫秒数。','傳送第一則訊息前隨機等待的實際毫秒數。'),
'presets.custom.action.chat.messageDelay':('Random real-time delay before each later message, in milliseconds.','发送后续每条消息前随机等待的实际毫秒数。','傳送後續每則訊息前隨機等待的實際毫秒數。'),
'presets.custom.action.chat.messages':('Messages sent in order; a leading slash sends a server command.','按顺序发送的消息；以斜杠开头时发送服务器命令。','依序傳送的訊息；以斜線開頭時傳送伺服器指令。'),
'presets.custom.control.enabled':('Control selected combat and movement modules while queueing.','排队期间控制所选战斗和移动模块。','排隊期間控制所選戰鬥與移動模組。'),
'presets.custom.control.killAura':('Disable KillAura on a trigger and enable it when the next untriggered check occurs.','触发时关闭杀戮光环，下次未触发检查时开启。','觸發時關閉殺戮光環，下次未觸發檢查時開啟。'),
'presets.custom.control.speed':('Disable Speed on a trigger and enable it when the next untriggered check occurs.','触发时关闭速度模块，下次未触发检查时开启。','觸發時關閉速度模組，下次未觸發檢查時開啟。'),
'presets.custom.waitUntilWorldChange':('Wait for an actual world replacement after the action, then wait twenty ticks.','动作完成后等待真实世界切换，再等待二十个游戏刻。','動作完成後等待實際世界切換，再等待二十個遊戲刻。'),
}
for prefix,verb_cn,verb_tw in [('presets.custom.trigger.item','用于触发排队','用於觸發排隊'),('presets.custom.action.useItem','用于选择并使用','用於選取並使用')]:
    TEXTS[prefix+'.mode']=('Match an item by its displayed hover name or registry id.',f'{verb_cn}的物品匹配方式：显示名称或注册标识。',f'{verb_tw}的物品比對方式：顯示名稱或登錄識別碼。')
    TEXTS[prefix+'.mode.name.names']=('Displayed item names to match; offhand is searched before the hotbar.','需要匹配的物品显示名称；先检查副手，再检查快捷栏。','必須比對的物品顯示名稱；先檢查副手，再檢查快捷列。')
    TEXTS[prefix+'.mode.name.mode']=('Choose exact, substring, prefix or suffix matching, with optional case folding.','选择相等、包含、开头或结尾匹配，可选择忽略大小写。','選擇相等、包含、開頭或結尾比對，可選擇忽略大小寫。')
    TEXTS[prefix+'.mode.item.items']=('Item registry ids to match, including empty air stacks when selected.','需要匹配的物品注册标识；选择空气时也会匹配空物品栏。','必須比對的物品登錄識別碼；選擇空氣時也會比對空物品欄。')
def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['AutoQueue']=LABELS['AutoQueue'][index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
        for suffix,values in TEXTS.items():
            key='liquidbounce.module.autoQueue'+('.'+suffix if suffix else '')+'.description';data['descriptions'][key]=values[index]
            if locale=='zh_cn':data['englishDescriptions'][key]=values[0]
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'AutoQueue descriptions')
if __name__=='__main__':main()
