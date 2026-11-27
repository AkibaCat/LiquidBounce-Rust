#!/usr/bin/env python3
"""Complete AntiBot Chinese display labels and explanatory resources."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'AntiBot':('反机器人','反機器人'),'LiteralNPC':('非在线玩家','非線上玩家'),'NotInTabList':('未列入玩家列表','未列入玩家清單'),'Grace':('宽限时间','寬限時間'),
 'Conditions':('判定条件','判定條件'),'InvalidGround':('异常落地','異常落地'),'VLToConsiderAsBot':('违规阈值','違規門檻'),
 'AlwaysInRadius':('始终位于范围内','始終位於範圍內'),'AlwaysInRadiusRange':('检测半径','偵測半徑'),'Age':('实体年龄','實體年齡'),'Minimum':('最小值','最小值'),
 'Moved':('移动记录','移動紀錄'),'Metadata':('元数据','中繼資料'),'Maximum':('最大值','最大值'),'Armor':('装备规则','裝備規則'),
 'Helmet':('头盔','頭盔'),'Chestplate':('胸甲','胸甲'),'Leggings':('护腿','護腿'),'Boots':('靴子','靴子'),'Name':('名字规则','名稱規則'),
 'Length':('长度范围','長度範圍'),'ValidateChars':('字符规则','字元規則'),'Duplicate':('重复名字','重複名稱'),'NoGameMode':('缺少游戏模式','缺少遊戲模式'),
 'Ping':('延迟','延遲'),'IllegalPitch':('异常俯仰角','異常俯仰角'),'FakeEntityID':('异常实体编号','異常實體編號'),'NeedHit':('必须被攻击','必須被攻擊'),
 'IllegalHealth':('异常生命值','異常生命值'),'Swung':('必须挥手','必須揮手'),'Critted':('必须产生暴击动画','必須產生暴擊動畫'),
 'Attributes':('必须收到属性','必須收到屬性'),'IllegalScale':('异常缩放','異常縮放'),'Nothing':('空装备槽','空裝備欄'),
 'Leather':('皮革','皮革'),'Chain':('锁链','鎖鏈'),'Iron':('铁','鐵'),'Gold':('金','金'),'Diamond':('钻石','鑽石'),'Netherite':('下界合金','獄髓'),
 'Elytra':('鞘翅或滑翔装备','鞘翅或滑翔裝備'),'TurtleScute':('海龟壳','海龜殼'),'Pumpkin':('雕刻南瓜','雕刻南瓜'),'Skull':('头颅','頭顱'),
 'Vanilla':('原版字符','原版字元'),'Cyrillic':('西里尔字符','西里爾字元'),'CJKUnifiedIdeographs':('中日韩统一表意文字','中日韓統一表意文字'),
 'Matrix-2021-10-18':('Matrix（2021-10-18）','Matrix（2021-10-18）'),'Intave-2022-07-28':('Intave（2022-07-28）','Intave（2022-07-28）'),'Horizon-2021-10-18':('Horizon（2021-10-18）','Horizon（2021-10-18）'),
}
TEXTS={
 '':('Identify player bots using the original four modes and ignore them in target tagging. Disabled by default.','按源码的四种模式识别玩家机器人，并在目标标记中忽略它们。默认关闭。','依原始碼的四種模式識別玩家機器人，並在目標標記中忽略它們。預設關閉。'),
 'Mode':('Select Custom, Matrix-2021-10-18, Intave-2022-07-28 or Horizon-2021-10-18. Default Custom.','选择自定义、Matrix、Intave 或 Horizon 模式。默认自定义。','選擇自訂、Matrix、Intave 或 Horizon 模式。預設自訂。'),
 'LiteralNPC':('Treat players missing from the online-player UUID collection as bots. Default false.','将 UUID 不在在线玩家集合中的玩家视为机器人，默认关闭。','將 UUID 不在線上玩家集合中的玩家視為機器人，預設關閉。'),
 'NotInTabList':('Check the server-listed player collection after a spawn grace period.','生成宽限期后检查服务端列入列表的玩家集合。','生成寬限期後檢查伺服器列入清單的玩家集合。'),
 'NotInTabList.Enabled':('Treat unlisted players as bots after Grace ticks. Default false.','经过宽限刻数后，将未列入玩家列表的玩家视为机器人，默认关闭。','經過寬限刻數後，將未列入玩家清單的玩家視為機器人，預設關閉。'),
 'NotInTabList.Grace':('Allow this many entity ticks before checking the listed player collection. Default 0; range 0–200.','检查玩家列表前允许经过的实体刻数，默认 0，范围 0–200。','檢查玩家清單前允許經過的實體刻數，預設 0，範圍 0–200。'),
 'Mode.Custom':('Combine the chosen checks and history-based groups; any enabled check can identify a bot.','组合所选条件和基于历史记录的规则；任一已启用规则成立即可识别机器人。','組合所選條件和依歷史紀錄的規則；任一已啟用規則成立即可識別機器人。'),
 'Mode.Custom.Conditions':('Select checks. Default NoGameMode, IllegalPitch and FakeEntityID.','选择判定条件，默认缺少游戏模式、异常俯仰角及异常实体编号。','選擇判定條件，預設缺少遊戲模式、異常俯仰角及異常實體編號。'),
 'Mode.Custom.InvalidGround':('Track inconsistent ground and movement state observed before vanilla packet handling.','记录原版数据包处理前观察到的落地与位移矛盾。','記錄原版封包處理前觀察到的落地與位移矛盾。'),
 'Mode.Custom.InvalidGround.Enabled':('Accumulate violations while grounded with a changed Y position; airborne packets halve them. Default true.','落地状态下 Y 坐标变化时累加违规值，离地数据包将其减半。默认开启。','落地狀態下 Y 座標變化時累加違規值，離地封包將其減半。預設開啟。'),
 'Mode.Custom.InvalidGround.VLToConsiderAsBot':('Violation count required to identify a bot. Default 10; range 1–50 flags.','识别机器人的违规值阈值，默认 10，范围 1–50。','識別機器人的違規值門檻，預設 10，範圍 1–50。'),
 'Mode.Custom.AlwaysInRadius':('Remember whether each player has ever left the configured radius.','记录玩家是否曾经离开所设半径。','記錄玩家是否曾經離開所設半徑。'),
 'Mode.Custom.AlwaysInRadius.Enabled':('Treat players that have never left the radius as bots. Default false.','将从未离开检测半径的玩家视为机器人，默认关闭。','將從未離開偵測半徑的玩家視為機器人，預設關閉。'),
 'Mode.Custom.AlwaysInRadius.AlwaysInRadiusRange':('Distance from the local player used for the persistent radius history. Default 20; range 5–30 blocks.','持续记录范围历史时使用的本地玩家距离，默认 20，范围 5–30 格。','持續記錄範圍歷史時使用的本機玩家距離，預設 20，範圍 5–30 格。'),
 'Mode.Custom.Age':('Check the player entity tick count.','检查玩家实体的存活刻数。','檢查玩家實體的存活刻數。'),
 'Mode.Custom.Age.Enabled':('Treat entities younger than Minimum as bots. Default false.','将存活时间小于最小值的实体视为机器人，默认关闭。','將存活時間小於最小值的實體視為機器人，預設關閉。'),
 'Mode.Custom.Age.Minimum':('Minimum entity age. Default 20; range 0–120 ticks.','最小实体年龄，默认 20，范围 0–120 刻。','最小實體年齡，預設 20，範圍 0–120 刻。'),
 'Mode.Custom.Moved':('Remember whether the player position has changed.','记录玩家位置是否变化过。','記錄玩家位置是否變化過。'),
 'Mode.Custom.Moved.Enabled':('After Minimum ticks, treat players without movement history as bots. Default false.','经过最小刻数后，将没有移动记录的玩家视为机器人，默认关闭。','經過最小刻數後，將沒有移動紀錄的玩家視為機器人，預設關閉。'),
 'Mode.Custom.Moved.Minimum':('Wait before requiring recorded movement. Default 160; range 20–400 ticks.','要求已有移动记录前等待的时间，默认 160，范围 20–400 刻。','要求已有移動紀錄前等待的時間，預設 160，範圍 20–400 刻。'),
 'Mode.Custom.Metadata':('Inspect only the first metadata packet for each entity.','仅检查每个实体的首个元数据包。','僅檢查每個實體的首個中繼資料封包。'),
 'Mode.Custom.Metadata.Enabled':('Treat a first metadata packet no larger than Maximum as a bot signal. Default false.','将首个元数据包条目数不超过最大值的实体视为机器人，默认关闭。','將首個中繼資料封包項目數不超過最大值的實體視為機器人，預設關閉。'),
 'Mode.Custom.Metadata.Maximum':('Maximum first-packet metadata entries considered minimal. Default 3; range 0–10 entries.','视为过少的首包元数据条目上限，默认 3，范围 0–10。','視為過少的首包中繼資料項目上限，預設 3，範圍 0–10。'),
 'Mode.Custom.Armor':('Remember players whose equipment fails the selected per-slot rules, clearing them once valid.','记录装备不符合各槽位所选规则的玩家，装备合规后清除。','記錄裝備不符合各欄位所選規則的玩家，裝備合規後清除。'),
 'Mode.Custom.Armor.Enabled':('Enable equipment-rule history. Default false.','启用装备规则历史，默认关闭。','啟用裝備規則歷史，預設關閉。'),
 'Mode.Custom.Armor.Helmet':('Accepted head equipment; empty selection skips the slot. Default Nothing. Includes turtle helmet, carved pumpkin and skulls.','允许的头部装备；不选择则跳过此槽位。默认空槽，可选海龟壳、雕刻南瓜及头颅。','允許的頭部裝備；不選擇則略過此欄位。預設空欄，可選海龜殼、雕刻南瓜及頭顱。'),
 'Mode.Custom.Armor.Chestplate':('Accepted chest equipment; empty selection skips the slot. Default Nothing. Elytra matches the glider component.','允许的胸部装备；不选择则跳过此槽位。默认空槽，鞘翅选项匹配滑翔组件。','允許的胸部裝備；不選擇則略過此欄位。預設空欄，鞘翅選項比對滑翔元件。'),
 'Mode.Custom.Armor.Leggings':('Accepted leg equipment, from Nothing through Netherite; empty selection skips the slot. Default Nothing.','允许的腿部装备，范围为空槽至下界合金；不选择则跳过此槽位。默认空槽。','允許的腿部裝備，範圍為空欄至獄髓；不選擇則略過此欄位。預設空欄。'),
 'Mode.Custom.Armor.Boots':('Accepted foot equipment, from Nothing through Netherite; empty selection skips the slot. Default Nothing.','允许的脚部装备，范围为空槽至下界合金；不选择则跳过此槽位。默认空槽。','允許的腳部裝備，範圍為空欄至獄髓；不選擇則略過此欄位。預設空欄。'),
 'Mode.Custom.Name':('Check scoreboard-name length and the selected character validators.','检查计分板名字的长度和所选字符规则。','檢查計分板名稱的長度和所選字元規則。'),
 'Mode.Custom.Name.Enabled':('Enable name validation. Default true.','启用名字验证，默认开启。','啟用名稱驗證，預設開啟。'),
 'Mode.Custom.Name.Length':('Accepted UTF-16 name length. Default 3–16; allowed endpoints 1–32.','允许的 UTF-16 名字长度，默认 3–16，两端允许范围 1–32。','允許的 UTF-16 名稱長度，預設 3–16，兩端允許範圍 1–32。'),
 'Mode.Custom.Name.ValidateChars':('Every selected validator must accept the entire name. Empty selection disables character checks. Default Vanilla.','每条所选规则都必须接受整个名字；不选择则关闭字符检查。默认原版字符。','每條所選規則都必須接受整個名稱；不選擇則關閉字元檢查。預設原版字元。'),
 'Mode.Matrix-2021-10-18':('Detect duplicate profiles or property-free suspects with fully enchanted armor or an equipment-object change after one tick.','检测重复档案，或无档案属性且穿齐附魔护甲、或等待一刻后装备对象变化的可疑玩家。','偵測重複個人資料，或無個人資料屬性且穿齊附魔盔甲、或等待一刻後裝備物件變化的可疑玩家。'),
 'Mode.Intave-2022-07-28':('Detect a suspect whose initial ping of at least 2 changes to zero within 15 ms. The first packet action controls handling.','检测初始延迟至少为 2、并在 15 毫秒内变为零的可疑玩家；按数据包的首个动作处理。','偵測初始延遲至少為 2、並在 15 毫秒內變為零的可疑玩家；依封包的首個動作處理。'),
 'Mode.Horizon-2021-10-18':('Detect added player-info entries whose game mode is absent.','检测新增玩家信息条目缺少游戏模式的情况。','偵測新增玩家資訊項目缺少遊戲模式的情況。'),
}
CONDITIONS={
 'Duplicate':('Exactly one online profile has the same name and a different UUID.','恰好一个在线档案名字相同、UUID 不同。','恰好一個線上個人資料名稱相同、UUID 不同。'),
 'NoGameMode':('Player info is missing or its game mode is null.','缺少玩家信息或其游戏模式为空。','缺少玩家資訊或其遊戲模式為空。'),
 'Ping':('The entity is at least 20 ticks old and its ping is zero or negative.','实体存活至少 20 刻且延迟为零或负数。','實體存活至少 20 刻且延遲為零或負數。'),
 'IllegalPitch':('Absolute player pitch exceeds 90 degrees.','玩家俯仰角的绝对值超过 90 度。','玩家俯仰角的絕對值超過 90 度。'),
 'FakeEntityID':('The entity ID is outside 0 through 1,000,000,000.','实体编号不在 0 至 1,000,000,000 之间。','實體編號不在 0 至 1,000,000,000 之間。'),
 'NeedHit':('No attack-entity event has targeted this entity.','尚无攻击实体事件以该实体为目标。','尚無攻擊實體事件以該實體為目標。'),
 'IllegalHealth':('The entity health exceeds the local player maximum health.','实体生命值超过本地玩家的最大生命值。','實體生命值超過本機玩家的最大生命值。'),
 'Swung':('No swing-animation packet has been observed.','尚未观察到挥手动画包。','尚未觀察到揮手動畫封包。'),
 'Critted':('No critical or magic-critical animation packet has been observed.','尚未观察到暴击或魔法暴击动画包。','尚未觀察到暴擊或魔法暴擊動畫封包。'),
 'Attributes':('No entity-attributes packet has been observed.','尚未观察到实体属性包。','尚未觀察到實體屬性封包。'),
 'IllegalScale':('The computed scale attribute is not exactly 1.0.','计算后的缩放属性不严格等于 1.0。','計算後的縮放屬性不嚴格等於 1.0。'),
}
for tag,text in CONDITIONS.items():
    TEXTS['Mode.Custom.Conditions.'+tag]=text
for tag in ['Nothing','Leather','Chain','Iron','Gold','Diamond','Netherite','Elytra','TurtleScute','Pumpkin','Skull']:
    cn,tw=LABELS[tag]
    TEXTS['Mode.Custom.Armor.'+tag]=(f'Accept {tag} equipment where this choice is available.',f'在提供此选项的槽位中接受{cn}。',f'在提供此選項的欄位中接受{tw}。')
for tag,text in {
 'Vanilla':('ASCII letters, digits and underscore.','ASCII 字母、数字和下划线。','ASCII 字母、數字和底線。'),
 'Cyrillic':('Code points U+0400 through U+052F.','码点 U+0400 至 U+052F。','碼點 U+0400 至 U+052F。'),
 'CJKUnifiedIdeographs':('Code points U+4E00 through U+9FA5.','码点 U+4E00 至 U+9FA5。','碼點 U+4E00 至 U+9FA5。'),
}.items():
    TEXTS['Mode.Custom.Name.ValidateChars.'+tag]=text


def description_key(path):
    suffix='.'.join(part[:1].lower()+part[1:] for part in path.split('.'))
    return 'liquidbounce.module.antiBot'+('.'+suffix if suffix else '')+'.description'


def update(data,index):
    data['moduleNames']['AntiBot']=LABELS['AntiBot'][index-1]
    data['labels'].update({key:text[index-1] for key,text in LABELS.items()})
    for path,text in TEXTS.items():
        key=description_key(path)
        data['descriptions'][key]=text[index]
        if index==1:
            data['englishDescriptions'][key]=text[0]
    return data


def main():
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=FOLDER/f'display_{locale}.json'
        data=update(json.loads(path.read_text(encoding='utf8')),index)
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        print(locale,len(TEXTS),'AntiBot descriptions')


if __name__=='__main__':
    main()
