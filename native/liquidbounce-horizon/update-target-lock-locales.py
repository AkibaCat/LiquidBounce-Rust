#!/usr/bin/env python3
"""TargetLock complete setting help and labels; run against integrated copies."""
from pathlib import Path
import argparse,json
ROOT=Path(__file__).resolve().parents[2];BASE='liquidbounce.module.targetLock.'
HELP={
'en_us':{
'mode.description':'Temporary locks players on attack until they leave range or time expires. Filter selects profile usernames. Temporary locks survive toggles and mode changes.',
'combat.description':'Restrict only combat targeting: other players become Interesting. When disabled, other players become Ignored for all targeting consumers.',
'mode.temporary.maximumTime.description':'Seconds before an attack lock expires. Repeated attacks refresh the deadline. Default 30; UI range 0–120 seconds.',
'mode.temporary.maximumRange.description':'Remove locks when a player is at or beyond this Euclidean distance. Range is checked before expiry. Default 20; UI range 8–40 blocks.',
'mode.temporary.whenNoLock.description':'AllowAll permits all players while no lock exists; AllowNone excludes every player until an attack creates a lock.',
'mode.filter.usernames.description':'Profile usernames matched with Java case-insensitive comparison. Entries preserve order and duplicates; null entries match no player.',
'mode.filter.filterType.description':'Whitelist permits listed usernames. Blacklist permits every username absent from the list.'},
'zh_cn':{
'mode.description':'临时模式在攻击玩家后锁定，超距或超时解除；筛选模式按游戏档案用户名筛选。临时锁定会保留至清理，不会因开关或切换模式直接清空。',
'combat.description':'开启时只限制战斗选敌，其他玩家仍作为“感兴趣”目标；关闭时将其他玩家标为“忽略”，影响所有使用目标分类的模块。',
'mode.temporary.maximumTime.description':'攻击锁定的有效秒数。重复攻击会刷新期限。默认 30 秒，界面范围 0～120 秒。',
'mode.temporary.maximumRange.description':'玩家达到或超过此直线距离时解除锁定，先检查距离再检查超时。默认 20 格，界面范围 8～40 格。',
'mode.temporary.whenNoLock.description':'允许全部：没有锁定时允许所有玩家；全部禁止：没有锁定时排除所有玩家，直到攻击产生锁定。',
'mode.filter.usernames.description':'按游戏档案用户名匹配，使用 Java 的忽略大小写规则。保留顺序与重复项，空值不匹配任何玩家。',
'mode.filter.filterType.description':'白名单只允许列表中的用户名；黑名单允许列表以外的所有用户名。'},
'zh_tw':{
'mode.description':'臨時模式在攻擊玩家後鎖定，超距或逾時解除；篩選模式按遊戲檔案使用者名稱篩選。臨時鎖定不會因開關或切換模式直接清空。',
'combat.description':'開啟時只限制戰鬥選敵，其他玩家仍作為「感興趣」目標；關閉時將其他玩家標為「忽略」，影響所有使用目標分類的模組。',
'mode.temporary.maximumTime.description':'攻擊鎖定的有效秒數。重複攻擊會刷新期限。預設 30 秒，介面範圍 0～120 秒。',
'mode.temporary.maximumRange.description':'玩家達到或超過此直線距離時解除鎖定，先檢查距離再檢查逾時。預設 20 格，介面範圍 8～40 格。',
'mode.temporary.whenNoLock.description':'允許全部：沒有鎖定時允許所有玩家；全部禁止：沒有鎖定時排除所有玩家，直到攻擊產生鎖定。',
'mode.filter.usernames.description':'按遊戲檔案使用者名稱匹配，使用 Java 的忽略大小寫規則。保留順序與重複項，空值不匹配任何玩家。',
'mode.filter.filterType.description':'白名單只允許清單中的使用者名稱；黑名單允許清單以外的所有使用者名稱。'}}
LABELS={'zh_cn':{'TargetLock':'目标锁定','Temporary':'临时','Filter':'筛选','Combat':'仅战斗','MaximumTime':'最长时间','MaximumRange':'最大距离','WhenNoLock':'未锁定时','AllowAll':'允许全部','AllowNone':'全部禁止','Usernames':'用户名','FilterType':'筛选类型','Whitelist':'白名单','Blacklist':'黑名单'},
'zh_tw':{'TargetLock':'目標鎖定','Temporary':'臨時','Filter':'篩選','Combat':'僅戰鬥','MaximumTime':'最長時間','MaximumRange':'最大距離','WhenNoLock':'未鎖定時','AllowAll':'允許全部','AllowNone':'全部禁止','Usernames':'使用者名稱','FilterType':'篩選類型','Whitelist':'白名單','Blacklist':'黑名單'}}
def update(language,target):
    data=json.loads(target.read_text(encoding='utf8'));source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
    data['descriptions'][BASE+'description']=source[BASE+'description'];data['descriptions'].update({BASE+k:v for k,v in HELP[language].items()});data['labels'].update(LABELS[language])
    if language=='zh_cn':
        en=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
        data['englishDescriptions'][BASE+'description']=en[BASE+'description'];data['englishDescriptions'].update({BASE+k:v for k,v in HELP['en_us'].items()})
    target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
def main():
    p=argparse.ArgumentParser();p.add_argument('--directory',default='native/liquidbounce-interop/resources/localization');a=p.parse_args()
    for language in ['zh_cn','zh_tw']:update(language,ROOT/a.directory/f'display_{language}.json')
if __name__=='__main__':main()
