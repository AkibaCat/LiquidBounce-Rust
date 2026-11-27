#!/usr/bin/env python3
"""Complete FastBreak help follows original tick, mode and disable behavior."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FOLDER=ROOT/'native/liquidbounce-interop/resources/localization'
TEXTS={
    'breakDamage':(
        'Each game tick, clear the block-breaking delay and complete progress only when it is strictly greater than this threshold. Default 0.8; the original slider range is 0.1–1.0. Disabling restores the normal delay of 5 ticks.',
        '每个游戏刻清除挖掘延迟，且仅当当前挖掘进度严格大于此阈值时将进度设为完成。默认 0.8，原始滑块范围为 0.1–1.0；关闭模块会恢复 5 刻延迟。',
        '每個遊戲刻清除挖掘延遲，且僅當目前挖掘進度嚴格大於此閾值時將進度設為完成。預設 0.8，原始滑桿範圍為 0.1–1.0；關閉模組會恢復 5 刻延遲。'),
    'onlyTool':(
        'Apply the tick changes and the Grim mode packet only while the actual main hand belongs to the axe, pickaxe, shovel or hoe item tags. Shears and swords do not qualify.',
        '仅当实际主手物品属于斧、镐、锹或锄的物品标签时应用挖掘进度、延迟修改及 Grim 模式数据包。剪刀与剑不满足条件。',
        '僅當實際主手物品屬於斧、鎬、鏟或鋤的物品標籤時套用挖掘進度、延遲修改及 Grim 模式封包。剪刀與劍不符合條件。'),
    'mode':(
        'None keeps only the tick changes. Grim-2.3.48 additionally sends an ordinary ABORT_DESTROY_BLOCK packet for the block above each outgoing STOP_DESTROY_BLOCK, preserving its face and using sequence 0. AbortAnother is the original mode alias.',
        'None 仅修改每刻的挖掘状态；Grim-2.3.48 还会在每个发出的 STOP_DESTROY_BLOCK 包前，向其上方方块发送普通 ABORT_DESTROY_BLOCK 包，保留朝向并使用序列号 0。原模式别名为 AbortAnother。',
        'None 僅修改每刻的挖掘狀態；Grim-2.3.48 還會在每個送出的 STOP_DESTROY_BLOCK 封包前，向其上方方塊傳送普通 ABORT_DESTROY_BLOCK 封包，保留方向並使用序號 0。原模式別名為 AbortAnother。'),
}
for locale,index in [('zh_cn',1),('zh_tw',2)]:
    path=FOLDER/f'display_{locale}.json'
    data=json.loads(path.read_text(encoding='utf-8'))
    for name,text in TEXTS.items():
        key=f'liquidbounce.module.fastBreak.{name}.description'
        data['descriptions'][key]=text[index]
        if locale=='zh_cn':data['englishDescriptions'][key]=text[0]
    path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'{locale}: {len(data["descriptions"])} owned descriptions')
