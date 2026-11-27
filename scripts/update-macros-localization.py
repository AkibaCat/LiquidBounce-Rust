#!/usr/bin/env python3
"""Add source-complete Macros and CommandManager Chinese display strings."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'Macros':('宏','巨集'),'CommandManager':('命令管理器','指令管理員'),'Commands':('命令','指令'),'Prefix':('命令前缀','指令前綴'),'HintCount':('提示数量','提示數量'),'TriggerDelay':('触发间隔','觸發間隔'),'Trigger':('触发按键','觸發按鍵'),'Delay':('消息间隔','訊息間隔'),'Messages':('消息列表','訊息清單'),'UseClientCommand':('执行客户端命令','執行用戶端指令'),'Action':('动作','動作'),'PickMode':('物品匹配方式','物品比對方式'),'Name':('名称','名稱'),'Names':('名称列表','名稱清單'),'Items':('物品列表','物品清單'),'SwingMode':('挥手显示','揮手顯示'),'HoldTime':('静默槽位保持时间','靜默欄位保持時間'),'UseOnly':('仅使用','僅使用'),'PlaceOrUse':('放置或使用','放置或使用'),'DoNotHide':('正常显示','正常顯示'),'HideForClient':('在本地隐藏','在本機隱藏')}
TEXTS={'liquidbounce.module.macros.description':('Execute chat sequences or item actions using five chat and five item key macros.','通过五个聊天宏和五个物品宏按键，执行聊天序列或物品动作。','透過五個聊天巨集與五個物品巨集按鍵，執行聊天序列或物品動作。'),'liquidbounce.commandManager.prefix.description':('Prefix that routes manually entered chat or enabled chat macros to client commands.','将手动聊天或启用命令功能的聊天宏路由至客户端命令的前缀。','將手動聊天或啟用指令功能的聊天巨集導向用戶端指令的前綴。'),'liquidbounce.commandManager.hintCount.description':('Number of nearby command names suggested for an unknown command.','输入未知命令时，按名称相似度显示的命令提示数量。','輸入未知指令時，依名稱相似度顯示的指令提示數量。')}
def add(path,en,cn,tw):TEXTS['liquidbounce.module.macros.'+path+'.description']=(en,cn,tw)
for kind in ['chat','item']:
    for i in range(1,6):
        path=f'{kind}-{i}';name=f'{kind.title()}-{i}'
        LABELS[name]=(f'聊天宏 {i}' if kind=='chat' else f'物品宏 {i}',f'聊天巨集 {i}' if kind=='chat' else f'物品巨集 {i}')
        add(path+'.triggerDelay','Random minimum elapsed milliseconds between triggers; the elapsed time must be strictly greater.','两次触发之间随机取样的最小毫秒数；实际间隔必须严格大于此值。','兩次觸發之間隨機取樣的最小毫秒數；實際間隔必須嚴格大於此值。')
        add(path+'.trigger','Keyboard key tested on press, repeat and release while no game or browser screen is open.','游戏及浏览器界面均关闭时，在按下、重复和松开事件中检查的键盘按键。','遊戲及瀏覽器介面均關閉時，在按下、重複和鬆開事件中檢查的鍵盤按鍵。')
        if kind=='chat':
            add(path+'.delay','Random real-time delay after every message, including the final message.','每条消息发送后随机等待的实际毫秒数，包括最后一条消息。','每則訊息傳送後隨機等待的實際毫秒數，包括最後一則訊息。')
            add(path+'.messages','Messages sent in list order. A leading slash sends a server command.','按列表顺序发送的消息；以斜杠开头时发送服务器命令。','依清單順序傳送的訊息；以斜線開頭時傳送伺服器指令。')
            add(path+'.useClientCommand','Execute a message beginning with the current client command prefix locally; command errors cancel this key event’s sequences.','消息以当前客户端命令前缀开头时在本地执行；命令出错会取消同一按键事件的序列。','訊息以目前用戶端指令前綴開頭時在本機執行；指令出錯會取消同一按鍵事件的序列。')
        else:
            add(path+'.action','Use the selected item directly, or attempt placement then use on a passed block interaction.','直接使用所选物品，或先尝试方块交互，在返回 Pass 后使用物品。','直接使用所選物品，或先嘗試方塊互動，在傳回 Pass 後使用物品。')
            add(path+'.pickMode','Match displayed hover names or item registry identifiers; search offhand before hotbar slots.','按显示名称或物品注册标识匹配；先搜索副手，再搜索快捷栏。','依顯示名稱或物品登錄識別碼比對；先搜尋副手，再搜尋快捷列。')
            add(path+'.pickMode.name.names','Displayed hover names to match; configured text is retained verbatim.','需要匹配的物品显示名称；保留原样设置文本。','需要比對的物品顯示名稱；保留原樣設定文字。')
            add(path+'.pickMode.name.mode','Choose exact, substring, prefix or suffix matching, with optional case folding.','选择相等、包含、开头或结尾匹配，可选择忽略大小写。','選擇相等、包含、開頭或結尾比對，可選擇忽略大小寫。')
            add(path+'.pickMode.item.items','Item registry identifiers, including empty slots when air is selected.','需要匹配的物品注册标识；选择空气时也会匹配空槽位。','需要比對的物品登錄識別碼；選擇空氣時也會比對空欄位。')
            add(path+'.swingMode','Show or hide the locally predicted interaction swing.','显示或隐藏本地预测的交互挥手动画。','顯示或隱藏本機預測的互動揮手動畫。')
            add(path+'.holdTime','Random ticks before this macro’s silent hotbar selection expires. The item action executes immediately.','此宏的静默快捷栏选择到期前随机保持的游戏刻数；物品动作立即执行。','此巨集的靜默快捷列選擇到期前隨機保持的遊戲刻數；物品動作立即執行。')
def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['Macros']=LABELS['Macros'][index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
        for key,values in TEXTS.items():
            data['descriptions'][key]=values[index]
            if locale=='zh_cn':data['englishDescriptions'][key]=values[0]
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'Macros/CommandManager descriptions')
if __name__=='__main__':main()
