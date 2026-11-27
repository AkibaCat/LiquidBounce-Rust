"""LogoffSpot nested Chinese display labels, descriptions and messages."""
from pathlib import Path
import argparse,json
KEY='liquidbounce.module.logoffSpot'
DESCRIPTIONS={'':('在玩家离开时保留原生假玩家，记录其最后位置。','在玩家離開時保留原生假玩家，記錄其最後位置。'),'SendInChat':('在玩家消失、重现或其区块卸载时显示本地聊天提示。','在玩家消失、重現或其區塊卸載時顯示本機聊天提示。'),'Bind':('设置切换下线位置的按键。','設定切換離線位置的按鍵。'),'Hidden':('在模块列表中隐藏下线位置。','在模組清單中隱藏離線位置。')}
MESSAGES={'disappeared':('%s disappeared at X: %s, Y: %s, Z: %s.','%s 在 X: %s，Y: %s，Z: %s 处消失。','%s 在 X: %s，Y: %s，Z: %s 處消失。'),'reappeared':('%s reappeared.','%s 重新出现。','%s 重新出現。'),'unloaded':('%s position was unloaded and is no longer visible.','%s 所在位置已卸载，不再显示。','%s 所在位置已卸載，不再顯示。')}
def update(language,path):
 d=json.loads(path.read_text(encoding='utf8'));i=0 if language=='zh_cn'else 1
 d.setdefault('moduleNames',{})['LogoffSpot']=('下线位置','離線位置')[i]
 d.setdefault('labels',{})['LogoffSpot']=('下线位置','離線位置')[i]
 d['labels']['SendInChat']=('聊天提示','聊天提示')[i]
 for name,values in DESCRIPTIONS.items():
  k=KEY+('.'+name[:1].lower()+name[1:]if name else'')+'.description';d.setdefault('descriptions',{})[k]=values[i];d.setdefault('englishDescriptions',{})[k]=f'LogoffSpot {name or "module"} setting.'
 for name,values in MESSAGES.items():d['descriptions'][KEY+'.messages.'+name]=values[i+1];d['englishDescriptions'][KEY+'.messages.'+name]=values[0]
 path.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--directory',required=True);args=p.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,Path(args.directory)/f'display_{lang}.json')
