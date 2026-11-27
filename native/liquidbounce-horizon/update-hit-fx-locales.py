"""HitFX display dictionaries only, preserving canonical locale identifiers."""
from pathlib import Path
import argparse,json
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'engine').is_dir()else HERE.parents[1];ROOT=NATIVE.parents[1]
PREFIX='liquidbounce.module.hitFX'
LABELS={'HitFX':'命中特效','ParticleAmount':'粒子数量','OtherSound':'目标受击音效','SelfSound':'自身受击音效','Blood':'血液','Fire':'火焰','Heart':'爱心','Water':'水滴','Smoke':'烟雾','Magic':'魔法','Crits':'暴击','Hit':'箭矢命中','Orb':'经验球','Bonk':'敲击','Boykisser':'Boykisser','ApplePay':'Apple Pay','Aimbooster':'Aimbooster','Bring':'铃声','Brick':'砖块','Bump':'碰撞','Glass':'玻璃','Click':'点击','Coin':'硬币','Meow':'喵','Moan':'呻吟','MagicSquash':'魔法挤压','NYA':'NYA','OSU':'OSU','Pop':'啪嗒','Soft':'柔和','Schoolboy':'Schoolboy','Skeet':'Skeet','Slap':'拍打','Squash':'挤压','Tung':'Tung','TF2 Crit':'TF2 暴击','UWU':'UWU'}
LABELS_TW={**LABELS,'HitFX':'命中特效','ParticleAmount':'粒子數量','OtherSound':'目標受擊音效','SelfSound':'自身受擊音效','Blood':'血液','Fire':'火焰','Heart':'愛心','Water':'水滴','Smoke':'煙霧','Magic':'魔法','Crits':'暴擊','Hit':'箭矢命中','Orb':'經驗球','Bonk':'敲擊','Bring':'鈴聲','Brick':'磚塊','Glass':'玻璃','Click':'點擊','Coin':'硬幣','Moan':'呻吟','MagicSquash':'魔法擠壓','Soft':'柔和','Slap':'拍打','Squash':'擠壓','TF2 Crit':'TF2 暴擊'}
HELP={'':'在命中确认声音附近为最近攻击且仍存活的目标生成粒子，并替换目标与自身的受击音效。','particle':'命中时从所选粒子中随机抽取；允许全部取消，默认火焰。','particleAmount':'每次确认命中创建的粒子发射器数量，默认 1～1；血液使用红石块破碎粒子。','otherSound':'从所选音效组及其变体随机选择，替换目标附近四格内的原版命中声音；允许留空，默认 Pop。','selfSound':'替换本地玩家受伤事件的声音；允许留空以使用原版声音，默认 Boykisser。'}
HELP_TW={'':'在命中確認聲音附近為最近攻擊且仍存活的目標產生粒子，並取代目標與自身的受擊音效。','particle':'命中時從選取的粒子中隨機抽取；允許全部取消，預設火焰。','particleAmount':'每次確認命中建立的粒子發射器數量，預設 1～1；血液使用紅石方塊破碎粒子。','otherSound':'從選取的音效組及其變體隨機選取，取代目標附近四格內的原版命中聲音；允許留空，預設 Pop。','selfSound':'取代本機玩家受傷事件的聲音；允許留空以使用原版聲音，預設 Boykisser。'}
EN={'':'Adds hit particles and original replacement sounds after a vanilla hit sound confirms a living last target within four blocks.','particle':'Randomly selects one of the enabled particle effects per emitter. Empty selection disables particles. Default Fire.','particleAmount':'Inclusive number of tracking emitters per confirmed hit, 1 to 20. Blood uses redstone block destruction effects.','otherSound':'Random sound group and variant replacing a recognized hit sound within four blocks of the last living target. Empty retains vanilla. Default Pop.','selfSound':'Replaces the local player damage-event hurt sound. Empty retains vanilla. Default Boykisser.'}
def descriptions(lang):return {PREFIX+('.'+k if k else'')+'.description':v for k,v in (HELP_TW if lang=='zh_tw'else EN if lang=='en_us'else HELP).items()}
def update(lang,target):
 d=json.loads(target.read_text(encoding='utf8'));d['labels'].update(LABELS_TW if lang=='zh_tw'else LABELS);d['descriptions'].update(descriptions(lang))
 if lang=='zh_cn':d['englishDescriptions'].update(descriptions('en_us'))
 target.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--directory',required=True);args=a.parse_args()
 for lang in ['zh_cn','zh_tw']:update(lang,ROOT/args.directory/f'display_{lang}.json')
