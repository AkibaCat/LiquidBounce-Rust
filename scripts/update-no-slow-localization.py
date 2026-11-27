#!/usr/bin/env python3
"""NoSlow simplified and traditional Chinese display strings; stable config IDs."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
LABELS={
 'Blocking':('格挡','格擋'),'Consume':('进食与饮用','進食與飲用'),'Bow':('弓与投掷武器','弓與投擲武器'),'Bundle':('收纳袋','收納袋'),'Sneaking':('潜行','潛行'),'Soulsand':('灵魂沙','靈魂沙'),'SlimeBlock':('黏液块','史萊姆方塊'),'HoneyBlock':('蜂蜜块','蜂蜜方塊'),'PowderSnow':('细雪','粉雪'),'Fluid':('流体','流體'),'Slowness':('缓慢效果','緩速效果'),
 'OnlySlowOnServerSide':('仅在服务器端减速','僅在伺服器端減速'),'NoBlockInteract':('阻止空方块交互','阻止空方塊互動'),'MinMultiplier':('最低速度倍率','最低速度倍率'),'PerLevelMultiplier':('每级减速倍率','每級減速倍率'),'Collision':('流体接触检测','流體接觸偵測'),
 'Reuse':('重新使用','重新使用'),'Switch':('切换','切換'),'Blink':('延迟发包','延遲傳送封包'),'Interact':('重新交互','重新互動'),'InvalidHand':('无效手部','無效手部'),'Release':('松开使用','鬆開使用'),'New':('新版','新版'),'Timing':('发送时机','傳送時機'),'PreAndPost':('更新前后','更新前後'),'Pre':('更新前','更新前'),'Post':('更新后','更新後')}
TEXTS={
 'noSlow':('Remove or configure slowdown from item use, sneaking, blocks, fluids and the Slowness effect.','分别处理物品使用、潜行、方块、流体和缓慢效果造成的减速。','分別處理物品使用、潛行、方塊、流體與緩速效果造成的減速。'),
 'noSlow.blocking.onlySlowOnServerSide':('Keep the original use multiplier while InteractionTracker reports blocking.','当交互跟踪器记录为格挡状态时，保留原本的使用减速倍率。','當互動追蹤器記錄為格擋狀態時，保留原本的使用減速倍率。'),
 'noSlow.sneaking.minMultiplier':('Minimum movement multiplier while sneaking. Default 1.0; range 0.3..1.0.','潜行时的最低移动倍率，默认 1.0，范围 0.3..1.0。','潛行時的最低移動倍率，預設 1.0，範圍 0.3..1.0。'),
 'noSlow.fluid.collision':('Keep normal fluid contact detection. Disabling this also cancels fluid membership checks.','保留正常的流体接触检测；关闭后会取消玩家的流体接触判断。','保留正常的流體接觸偵測；關閉後會取消玩家的流體接觸判斷。'),
 'noSlow.slowness.perLevelMultiplier':('Movement-speed reduction per Slowness level. Default 0; vanilla 0.15 is restored on disable.','每一级缓慢效果的移动速度减幅，默认 0；关闭后恢复原版的 0.15。','每一級緩速效果的移動速度減幅，預設 0；關閉後恢復原版的 0.15。'),
 'noSlow.consume.mode.intave-14.8.4.mode':('Release sends release packets and ends client use at tick three. New sends releases during the first two ticks and when use ends.','松开使用模式在使用第 3 刻停止客户端使用；新版在最初 2 刻及使用结束时发送松开使用请求。','鬆開使用模式在使用第 3 刻停止用戶端使用；新版在最初 2 刻及使用結束時傳送鬆開使用請求。'),
}
GROUPS=[('blocking','格挡','格擋'),('consume','进食与饮用','進食與飲用'),('bow','弓与投掷武器','弓與投擲武器'),('bundle','收纳袋','收納袋'),('sneaking','潜行','潛行'),('soulsand','灵魂沙','靈魂沙'),('slimeBlock','黏液块','史萊姆方塊'),('honeyBlock','蜂蜜块','蜂蜜方塊'),('powderSnow','细雪','粉雪'),('fluid','流体','流體'),('slowness','缓慢效果','緩速效果')]
def main():
 texts=dict(TEXTS)
 for i,(key,cn,tw)in enumerate(GROUPS):
  texts[f'noSlow.{key}.enabled']=(f'Enable NoSlow for {key}.',f'启用{cn}的减速处理。',f'啟用{tw}的減速處理。')
  if i<4:
   for field,eng,sc,tc in [('forward','forward','前后','前後'),('sideways','sideways','左右','左右')]:
    texts[f'noSlow.{key}.{field}']=(f'Replace the {eng} item-use multiplier. Default 1.0; range 0..1.',f'使用物品时的{sc}移动倍率，默认 1.0，范围 0..1。',f'使用物品時的{tc}移動倍率，預設 1.0，範圍 0..1。')
  if i in (1,2):texts[f'noSlow.{key}.noBlockInteract.enabled']=('Cancel item-on-block packets that target air while the matching item-use group is active.','对应的物品使用组生效时，取消目标为空气方块的交互请求。','對應的物品使用群組生效時，取消目標為空氣方塊的互動請求。')
  if 5<=i<=8:texts[f'noSlow.{key}.multiplier']=(f'Horizontal velocity multiplier for {key}. Default 1.0; range 0.4..2.0.',f'{cn}的水平速度倍率，默认 1.0，范围 0.4..2.0。',f'{tw}的水平速度倍率，預設 1.0，範圍 0.4..2.0。')
 for group,mode in [('blocking','reuse'),('blocking','switch'),('sneaking','switch'),('sneaking','aac-5')]:
  texts[f'noSlow.{group}.mode.{mode}.timing']=('Select before, after, or split across both sides of the network movement update.','选择在网络移动更新前、更新后，或分在更新前后发送请求。','選擇在網路移動更新前、更新後，或分在更新前後傳送請求。')
 for locale,index in [('zh_cn',1),('zh_tw',2)]:
  path=FOLDER/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'));data['moduleNames']['NoSlow']=('无减速','無減速')[index-1];data['labels'].update({k:v[index-1]for k,v in LABELS.items()})
  for suffix,strings in texts.items():
   key=f'liquidbounce.module.{suffix}.description';data['descriptions'][key]=strings[index]
   if locale=='zh_cn':data['englishDescriptions'][key]=strings[0]
  path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(locale,len(data['descriptions']))
if __name__=='__main__':main()
