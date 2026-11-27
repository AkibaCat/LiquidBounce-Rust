#!/usr/bin/env python3
"""MurderMystery labels, settings and descriptions in both game Chinese locales."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1];FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={'MurderMystery':('密室杀手','密室殺手'),'SetTeamPrefix':('设置队伍前缀','設定隊伍前綴'),'Classic':('经典','經典'),'Infection':('感染','感染'),'Assassination':('刺杀','刺殺')}
TEXTS={
 'liquidbounce.module.murderMystery.description':('Identify murderers, bow holders and contract targets across the Classic, Infection and Assassination modes.','在经典、感染和刺杀模式中识别凶手、持弓者、刺客及契约目标。','在經典、感染與刺殺模式中識別兇手、持弓者、刺客及契約目標。'),
 'liquidbounce.module.murderMystery.setTeamPrefix.description':('Replace the existing scoreboard team prefix with the detected player role.','将已有的计分板队伍前缀替换为识别到的角色；禁用模块后保留已写入的前缀。','將既有的計分板隊伍前綴替換為識別到的角色；停用模組後保留已寫入的前綴。'),
 'liquidbounce.module.murderMystery.mode.description':('Choose Classic equipment detection, Infection survivor detection, or Assassination contract and warning-sound detection.','选择经典装备检测、感染幸存者检测，或刺杀契约地图与警告声音检测。','選擇經典裝備偵測、感染倖存者偵測，或刺殺契約地圖與警告聲音偵測。'),
 'liquidbounce.module.murderMystery.mode.classic.description':('Remember sword holders as murderers and bow holders as detective-like players until reset.','在重置前记录持有凶器的凶手和持弓者，并按本地角色筛选攻击目标。','在重設前記錄持有凶器的兇手與持弓者，並依本機角色篩選攻擊目標。'),
 'liquidbounce.module.murderMystery.mode.infection.description':('Detect survivors holding arrows or using bows, and report only the first infected. Arrow dodging is suppressed while the local hand role is detective-like.','检测手持箭或正在使用弓的幸存者，仅提示首位感染者；本地手持角色为持弓者时禁止自动躲箭。','偵測手持箭或正在使用弓的倖存者，僅提示首位感染者；本機手持角色為持弓者時禁止自動閃箭。'),
 'liquidbounce.module.murderMystery.mode.assassination.description':('Read the contract map in hotbar slot four and infer the assassin from note-block bass-drum warning volume.','读取快捷栏第 4 槽中的契约地图，并根据音符盒低音鼓警告音量推算刺客。','讀取快捷列第 4 欄中的契約地圖，並依音符盒低音鼓警告音量推算刺客。'),
}
def main():
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
    path=FOLDER/f'display_{locale}.json';d=json.loads(path.read_text(encoding='utf8'));d['moduleNames']['MurderMystery']=LABELS['MurderMystery'][index-1];d['labels'].update({k:v[index-1]for k,v in LABELS.items()})
    for k,v in TEXTS.items():
        d['descriptions'][k]=v[index]
        if locale=='zh_cn':d['englishDescriptions'][k]=v[0]
    path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(TEXTS),'MurderMystery descriptions')
if __name__=='__main__':main()
