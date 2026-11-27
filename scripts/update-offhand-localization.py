#!/usr/bin/env python3
"""Complete Offhand Chinese labels, original setting paths and constraints."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
LABELS = {
    'SwitchMode': ('切换方式', '切換方式'), 'SwitchDelay': ('切换延时', '切換延遲'),
    'Cycle': ('轮换按键', '輪換按鍵'), 'Totem': ('不死图腾', '不死圖騰'),
    'Crystal': ('末地水晶', '終界水晶'), 'Gapple': ('金苹果', '金蘋果'),
    'StrengthPotion': ('力量药水', '力量藥水'), 'Block': ('方块', '方塊'),
    'Health': ('生命值条件', '生命值條件'), 'HealthThreshold': ('生命值阈值', '生命值門檻'),
    'Safety': ('安全位置', '安全位置'), 'SafeHealthThreshold': ('安全生命值阈值', '安全生命值門檻'),
    'SubtractCalculatedDamage': ('扣除预测伤害', '扣除預測傷害'),
    'PredictExplosionDamageEntities': ('预测实体爆炸', '預測實體爆炸'),
    'PredictExplosionDamageBlocks': ('预测方块爆炸', '預測方塊爆炸'),
    'MissingArmor': ('缺少护甲时启用', '缺少盔甲時啟用'),
    'PredictFallDamage': ('预测坠落伤害', '預測墜落傷害'),
    'IgnoreElytra': ('忽略鞘翅飞行', '忽略鞘翅飛行'), 'SwitchBack': ('允许切回', '允許切回'),
    'SwitchBackDelay': ('切回延时', '切回延遲'), 'MaceDive': ('重锤俯冲', '重錘俯衝'),
    'HorizontalRange': ('水平范围', '水平範圍'), 'MinFallSpeed': ('最小下降速度', '最小下降速度'),
    'Urgency': ('紧急度', '緊急度'), 'MinFactor': ('最小延时比例', '最小延遲比例'),
    'SendDirectly': ('直接发送', '直接傳送'), 'OnlyWhileCrystalAura': ('仅水晶光环运行时', '僅水晶光環執行時'),
    'WhenNoTotems': ('无图腾时替代', '無圖騰時替代'), 'CrystalBind': ('水晶按键', '水晶按鍵'),
    'GappleBind': ('金苹果按键', '金蘋果按鍵'), 'WhileHoldingSword': ('持剑时', '持劍時'),
    'OnlyWhileKillAura': ('仅杀戮光环运行时', '僅殺戮光環執行時'),
    'OnlyWhileHoldingSword': ('仅持剑时', '僅持劍時'), 'StrengthBind': ('力量药水按键', '力量藥水按鍵'),
    'WhileScaffold': ('搭路启用时', '搭路啟用時'), 'WhileEagle': ('边缘潜行启用时', '邊緣潛行啟用時'),
    'Smart': ('智能', '智慧'), 'Switch': ('交换', '交換'), 'PickUp': ('拾取', '拾取'),
    'Automatic': ('自动', '自動'),
}

# Source paths retain their original capitalization here; keys use lower camel case.
TEXTS = {
    '': ('Choose the offhand item from health, combat and building conditions.', '根据生命值、战斗和搭路条件选择副手物品。', '根據生命值、戰鬥與搭路條件選擇副手物品。'),
    'SwitchMode': ('Smart sends hotbar swap packets; Switch uses one swap click; PickUp uses pickup clicks. Automatic selects by protocol. Native default Switch.', '智能模式发送快捷栏交换数据包；交换模式点击一次；拾取模式依次拾取。自动模式按协议选择。原生默认交换。', '智慧模式傳送快捷列交換封包；交換模式點擊一次；拾取模式依次拾取。自動模式依通訊協定選擇。原生預設交換。'),
    'SwitchDelay': ('Delay between non-totem switches in milliseconds; default 0–0.', '非图腾切换之间的毫秒延时，默认 0–0。', '非圖騰切換之間的毫秒延遲，預設 0–0。'),
    'Cycle': ('Cycle through enabled selectable modes on key press. Default H.', '按下时轮换可选且启用的模式，默认 H。', '按下時輪換可選且啟用的模式，預設 H。'),
    'Totem': ('Equip death-protection items before other modes when danger requires it.', '危险条件满足时优先装备带死亡保护组件的物品。', '危險條件滿足時優先裝備帶死亡保護元件的物品。'),
    'Totem.Enabled': ('Enable the totem mode; default true.', '启用图腾模式，默认开启。', '啟用圖騰模式，預設開啟。'),
    'Totem.SwitchDelay': ('Totem-switch delay in milliseconds, scaled by urgency. Default 0–0.', '切入图腾的毫秒延时，可由紧急度缩短，默认 0–0。', '切入圖騰的毫秒延遲，可由緊急度縮短，預設 0–0。'),
    'Totem.SwitchBackDelay': ('Wait this many milliseconds when leaving totem mode. Default 40.', '离开图腾模式前等待的毫秒数，默认 40。', '離開圖騰模式前等待的毫秒數，預設 40。'),
    'Totem.Health': ('Use health, armor, explosion and fall conditions. Disabling this group requests a totem continuously.', '使用生命值、护甲、爆炸和坠落条件；关闭此组时持续请求图腾。', '使用生命值、盔甲、爆炸和墜落條件；關閉此組時持續請求圖騰。'),
    'Totem.Health.Enabled': ('Enable health conditions; default true.', '启用生命值条件，默认开启。', '啟用生命值條件，預設開啟。'),
    'Totem.Health.HealthThreshold': ('Equip a totem at or below this health plus absorption. Default 14.', '生命值与伤害吸收之和不高于此值时装备图腾，默认 14。', '生命值與傷害吸收之和不高於此值時裝備圖騰，預設 14。'),
    'Totem.Health.Safety': ('Use the safe threshold in a resistant burrow or a one-block hole.', '位于抗爆方块内或一格抗爆洞内时使用安全阈值。', '位於抗爆方塊內或一格抗爆洞內時使用安全門檻。'),
    'Totem.Health.Safety.Enabled': ('Enable safe-position health adjustment; default true.', '启用安全位置生命值调整，默认开启。', '啟用安全位置生命值調整，預設開啟。'),
    'Totem.Health.Safety.SafeHealthThreshold': ('Health threshold in a resistant burrow or hole. Default 10.', '抗爆掩体或洞内使用的生命值阈值，默认 10。', '抗爆掩體或洞內使用的生命值門檻，預設 10。'),
    'Totem.Health.SubtractCalculatedDamage': ('Subtract predicted damage before testing the health threshold. Otherwise only lethal predicted damage triggers. Default false.', '先扣除预测伤害再比较生命值阈值；关闭时预测伤害必须致命才触发。默认关闭。', '先扣除預測傷害再比較生命值門檻；關閉時預測傷害必須致命才觸發。預設關閉。'),
    'Totem.Health.PredictExplosionDamageEntities': ('Predict crystals, TNT, TNT minecarts and creepers. Use the largest explosion, then add fall damage. Default true.', '预测水晶、TNT、TNT 矿车和苦力怕，取最大爆炸伤害后加上坠落伤害。默认开启。', '預測水晶、TNT、TNT 礦車與苦力怕，取最大爆炸傷害後加上墜落傷害。預設開啟。'),
    'Totem.Health.PredictExplosionDamageBlocks': ('Predict exploding beds and charged anchors within radius 10. The source initializes its sphere only when this setting changes. Default false.', '预测半径 10 内会爆炸的床和已充能重生锚；源码仅在此设置变化时初始化搜索球。默认关闭。', '預測半徑 10 內會爆炸的床與已充能重生錨；原始碼僅在此設定變化時初始化搜尋球。預設關閉。'),
    'Totem.Health.MissingArmor': ('Request a totem while any armor slot is empty. Default true.', '任一护甲槽为空时请求图腾，默认开启。', '任一盔甲欄位為空時請求圖騰，預設開啟。'),
    'Totem.Health.PredictFallDamage': ('Predict the landing block for 20 ticks and calculate damage from current fall distance.', '向前模拟 20 刻获取落点，并按当前坠落距离计算伤害。', '向前模擬 20 刻取得落點，並依目前墜落距離計算傷害。'),
    'Totem.Health.PredictFallDamage.Enabled': ('Enable fall prediction unless NoFall is running. Default true.', '启用坠落预测；无摔落伤害模块运行时跳过。默认开启。', '啟用墜落預測；無摔落傷害模組執行時略過。預設開啟。'),
    'Totem.Health.PredictFallDamage.IgnoreElytra': ('Ignore fall damage while actually fall flying in the fall-flying pose. Default false.', '实际使用鞘翅且处于滑翔姿势时忽略坠落伤害，默认关闭。', '實際使用鞘翅且處於滑翔姿勢時忽略墜落傷害，預設關閉。'),
    'Totem.Health.SwitchBack': ('Remember the last source slot and allow the Back mode after a totem. Default true.', '记住上次来源槽位，并允许图腾后使用切回模式。默认开启。', '記住上次來源欄位，並允許圖騰後使用切回模式。預設開啟。'),
    'Totem.MaceDive': ('Equip a totem for a target diving from above with a mace.', '目标持重锤从上方俯冲时装备图腾。', '目標持重錘從上方俯衝時裝備圖騰。'),
    'Totem.MaceDive.Enabled': ('Enable mace-dive detection; default false.', '启用重锤俯冲检测，默认关闭。', '啟用重錘俯衝偵測，預設關閉。'),
    'Totem.MaceDive.HorizontalRange': ('Maximum horizontal distance of a diving target. Default 4 blocks.', '俯冲目标的最大水平距离，默认 4 格。', '俯衝目標的最大水平距離，預設 4 格。'),
    'Totem.MaceDive.MinFallSpeed': ('Minimum vertical displacement per tick while descending. Default 0.3 blocks/tick.', '每刻最小下降位移，默认 0.3 格。', '每刻最小下降位移，預設 0.3 格。'),
    'Totem.Urgency': ('Shorten totem delay using health, predicted damage and the nearest valid enemy.', '根据生命值、预测伤害和最近有效敌人缩短图腾延时。', '根據生命值、預測傷害與最近有效敵人縮短圖騰延遲。'),
    'Totem.Urgency.Enabled': ('Enable urgency scaling; default false.', '启用紧急度缩放，默认关闭。', '啟用緊急度縮放，預設關閉。'),
    'Totem.Urgency.MinFactor': ('Fraction of delay retained at maximum danger. Default 0.', '最高危险程度保留的延时比例，默认 0。', '最高危險程度保留的延遲比例，預設 0。'),
    'Totem.Urgency.Range': ('Enemy search radius measured from the local eye to target boxes. Default 6 blocks.', '从自身眼部到目标包围盒计算的敌人搜索半径，默认 6 格。', '從自身眼部到目標包圍盒計算的敵人搜尋半徑，預設 6 格。'),
    'Totem.SendDirectly': ('Perform totem clicks immediately, bypassing inventory requests and their constraints. Default false.', '立即执行图腾点击，跳过库存请求及其限制。默认关闭。', '立即執行圖騰點擊，略過庫存請求及其限制。預設關閉。'),
    'Crystal': ('Selectable end-crystal mode and fallback when no totems exist.', '可手动选择的水晶模式，也可在无图腾时替代。', '可手動選擇的水晶模式，也可在無圖騰時替代。'),
    'Crystal.Enabled': ('Enable crystal mode; default true.', '启用水晶模式，默认开启。', '啟用水晶模式，預設開啟。'),
    'Crystal.OnlyWhileCrystalAura': ('Allow selecting crystal mode only while CrystalAura is running. Default false.', '仅水晶光环运行时允许选择水晶模式，默认关闭。', '僅水晶光環執行時允許選擇水晶模式，預設關閉。'),
    'Crystal.WhenNoTotems': ('Use a crystal when totem mode finds no death-protection item. Default true.', '图腾模式找不到死亡保护物品时使用水晶，默认开启。', '圖騰模式找不到死亡保護物品時使用水晶，預設開啟。'),
    'Crystal.CrystalBind': ('Select crystal mode; pressing again can restore the previous selectable mode. Default unbound.', '选择水晶模式；再次按下可恢复之前可选的模式。默认未绑定。', '選擇水晶模式；再次按下可恢復之前可選的模式。預設未綁定。'),
    'Gapple': ('Prefer enchanted golden apples, falling back to golden apples.', '优先使用附魔金苹果，否则使用普通金苹果。', '優先使用附魔金蘋果，否則使用普通金蘋果。'),
    'Gapple.Enabled': ('Enable golden-apple mode; default true.', '启用金苹果模式，默认开启。', '啟用金蘋果模式，預設開啟。'),
    'Gapple.GappleBind': ('Select golden-apple mode, with priority over other equal binds. Default unbound.', '选择金苹果模式；按键冲突时优先于其他模式。默认未绑定。', '選擇金蘋果模式；按鍵衝突時優先於其他模式。預設未綁定。'),
    'Gapple.WhileHoldingSword': ('Request a golden apple while holding a sword and meeting the combat gate.', '持剑且满足战斗条件时请求金苹果。', '持劍且滿足戰鬥條件時請求金蘋果。'),
    'Gapple.WhileHoldingSword.Enabled': ('Enable automatic golden apples while holding a sword. Default true.', '启用持剑时自动选择金苹果，默认开启。', '啟用持劍時自動選擇金蘋果，預設開啟。'),
    'Gapple.WhileHoldingSword.OnlyWhileKillAura': ('Require KillAura to be running for automatic sword-based selection. Default true.', '持剑自动选择时要求杀戮光环运行，默认开启。', '持劍自動選擇時要求殺戮光環執行，預設開啟。'),
    'StrengthPotion': ('Use a drinkable strength potion when its effect is absent.', '没有力量效果时使用可饮用的力量药水。', '沒有力量效果時使用可飲用的力量藥水。'),
    'StrengthPotion.Enabled': ('Enable strength-potion mode; default false.', '启用力量药水模式，默认关闭。', '啟用力量藥水模式，預設關閉。'),
    'StrengthPotion.OnlyWhileHoldingSword': ('Require a sword in the main hand. Default true.', '要求主手持剑，默认开启。', '要求主手持劍，預設開啟。'),
    'StrengthPotion.OnlyWhileKillAura': ('Require KillAura to be running. Default true.', '要求杀戮光环运行，默认开启。', '要求殺戮光環執行，預設開啟。'),
    'StrengthPotion.StrengthBind': ('Original strength bind; the source mode cannot be selected by cycling. Default unbound.', '源码力量药水按键；该模式不属于可轮换选择的模式。默认未绑定。', '原始碼力量藥水按鍵；此模式不屬於可輪換選擇的模式。預設未綁定。'),
    'Block': ('Equip a valid Scaffold block when the selected building module is enabled.', '选定搭路相关模块启用时装备符合搭路策略的方块。', '選定搭路相關模組啟用時裝備符合搭路策略的方塊。'),
    'Block.Enabled': ('Enable block mode; default false.', '启用方块模式，默认关闭。', '啟用方塊模式，預設關閉。'),
    'Block.WhileScaffold': ('Request a block while Scaffold is enabled. Default true.', '自动搭路启用时请求方块，默认开启。', '自動搭路啟用時請求方塊，預設開啟。'),
    'Block.WhileEagle': ('Request a block while Eagle is enabled. Default true.', '边缘潜行启用时请求方块，默认开启。', '邊緣潛行啟用時請求方塊，預設開啟。'),
}


def description_key(path):
    suffix = '.'.join(part[:1].lower() + part[1:] for part in path.split('.'))
    return 'liquidbounce.module.offhand' + ('.' + suffix if suffix else '') + '.description'


def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        data['moduleNames']['Offhand'] = ('自动副手', '自動副手')[index - 1]
        data['labels'].update({k: v[index - 1] for k, v in LABELS.items()})
        for field in ['descriptions', 'englishDescriptions']:
            if field not in data:
                continue
            for key, text in list(data[field].items()):
                if key.startswith('liquidbounce.module.elytraSwap.constraints'):
                    data[field][key.replace('.elytraSwap.', '.offhand.')] = text
        for setting, text in TEXTS.items():
            key = description_key(setting)
            data['descriptions'][key] = text[index]
            if locale == 'zh_cn':
                data['englishDescriptions'][key] = text[0]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
        print(locale, len(TEXTS), 'Offhand descriptions plus full constraints')


if __name__ == '__main__':
    main()
