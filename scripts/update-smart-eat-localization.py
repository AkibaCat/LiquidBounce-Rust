#!/usr/bin/env python3
"""Owned help for the complete pinned SmartEat configuration tree."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
HELP = {
    'swapBackDelay': (
        'Game ticks before restoring the previous hotbar slot. The first SilentOffhand selection lasts at least five ticks; continued eating or drinking renews this delay.',
        '恢复原快捷栏槽位前等待的游戏刻数。SilentOffhand 首次选槽至少保持 5 刻；持续进食或饮用时会续期。',
        '還原原快捷列欄位前等待的遊戲刻數。SilentOffhand 首次選欄至少保持 5 刻；持續進食或飲用時會延長時間。'),
    'preferGappleHealthThreshold': (
        'At or below this health, prefer regular golden apples. Lower active health thresholds rank first, followed by nutrition and slot proximity.',
        '生命值不高于此值时，优先选择普通金苹果。已满足的生命值条件中，阈值较低者优先，其次比较营养值和槽位距离。',
        '生命值不高於此值時，優先選取普通金蘋果。已滿足的生命值條件中，閾值較低者優先，其次比較營養值和欄位距離。'),
    'preferNotchAppleHealthThreshold': (
        'At or below this health, prefer enchanted golden apples. Lower active health thresholds rank first, followed by nutrition and slot proximity.',
        '生命值不高于此值时，优先选择附魔金苹果。已满足的生命值条件中，阈值较低者优先，其次比较营养值和槽位距离。',
        '生命值不高於此值時，優先選取附魔金蘋果。已滿足的生命值條件中，閾值較低者優先，其次比較營養值和欄位距離。'),
    'preferHealthPotHealthThreshold': (
        'At or below this health, consider drinkable potions containing Instant Health. Splash potions are excluded from this preference.',
        '生命值不高于此值时，将含瞬间治疗效果的饮用型药水加入优先候选；此规则不包含喷溅型药水。',
        '生命值不高於此值時，將含立即治療效果的飲用型藥水加入優先候選；此規則不包含飛濺型藥水。'),
    'combatPauseTime': (
        'Pause combat for at least this many game ticks when selecting food, eating, or forcing the use key.',
        '选取食物、进食或强制按住使用键时，暂停战斗至少指定的游戏刻数。',
        '選取食物、進食或強制按住使用鍵時，暫停戰鬥至少指定的遊戲刻數。'),
    'notDuringCombat': (
        'Prevent SilentOffhand actions and new AutoEat sequences during combat. An AutoEat sequence already running checks its normal completion condition.',
        '战斗期间不触发 SilentOffhand，也不启动新的 AutoEat 流程。已经开始的自动进食仍按原有条件判断何时完成。',
        '戰鬥期間不觸發 SilentOffhand，也不啟動新的 AutoEat 流程。已經開始的自動進食仍依原有條件判斷何時完成。'),
    'silentOffhand': (
        'Select food silently when a right-click with a mining tool would otherwise pass. The normal item-use path consumes the selected food.',
        '手持采掘工具且右键操作未被处理时，静默选取食物，由正常物品使用流程使用所选食物。',
        '手持挖掘工具且右鍵操作未被處理時，靜默選取食物，由正常物品使用流程使用所選食物。'),
    'silentOffhand.enabled': (
        'When a mining tool is held and right-click otherwise passes, silently select eligible food from the offhand or hotbar. This selection does not move items into the offhand.',
        '手持采掘工具且右键操作未被处理时，从副手或快捷栏中静默选取当前可食用的候选物品；选槽不会把物品移入副手。',
        '手持挖掘工具且右鍵操作未被處理時，從副手或快捷列中靜默選取目前可食用的候選物品；選欄不會把物品移入副手。'),
    'silentOffhand.renderSlot.enabled': (
        'Draw the best current food in a separate offhand-style HUD slot. It is a preview of the selected candidate.',
        '在独立的副手样式 HUD 槽位中显示当前最合适的食物，预览候选物品。',
        '在獨立的副手樣式 HUD 欄位中顯示目前最合適的食物，預覽候選物品。'),
    'silentOffhand.renderSlot': (
        'Preview the best food in an extra HUD slot left of the hotbar.',
        '在快捷栏左侧的额外 HUD 槽位中预览最合适的食物。',
        '在快捷列左側的額外 HUD 欄位中預覽最合適的食物。'),
    'silentOffhand.renderSlot.offset': (
        'Horizontal distance added to the left of the hotbar for the food preview. Larger values move it farther left.',
        '食物预览在快捷栏左侧额外偏移的距离。数值越大，预览越靠左。',
        '食物預覽在快捷列左側額外偏移的距離。數值越大，預覽越靠左。'),
    'autoEat.enabled': (
        'Start eating when hunger falls below MinHunger. Select eligible food and hold the use key until hunger rises above MinHunger.',
        '饥饿值低于 MinHunger 时开始自动进食，选择可用食物并按住使用键，直到饥饿值高于 MinHunger。',
        '飢餓值低於 MinHunger 時開始自動進食，選取可用食物並按住使用鍵，直到飢餓值高於 MinHunger。'),
    'autoEat': (
        'Automatically select and use food when hunger falls below the configured threshold, until it rises above that threshold.',
        '饥饿值低于设定阈值时，自动选择并使用食物，直到饥饿值高于阈值。',
        '飢餓值低於設定閾值時，自動選取並使用食物，直到飢餓值高於閾值。'),
    'autoEat.minHunger': (
        'Hunger threshold for AutoEat: strictly below starts a sequence; strictly above finishes it. Equality alone neither starts nor finishes a sequence.',
        '自动进食的饥饿值阈值：严格低于时开始，严格高于时结束。恰好等于阈值不会单独触发开始或结束。',
        '自動進食的飢餓值閾值：嚴格低於時開始，嚴格高於時結束。剛好等於閾值不會單獨觸發開始或結束。'),
}


def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        display = json.loads(path.read_text(encoding='utf-8'))
        for key, texts in HELP.items():
            key = 'liquidbounce.module.smartEat.' + key + '.description'
            display['descriptions'][key] = texts[index]
            if locale == 'zh_cn':
                display['englishDescriptions'][key] = texts[0]
        path.write_text(json.dumps(display, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        print(f'{locale}: {len(display["descriptions"])} owned descriptions')


if __name__ == '__main__':
    main()
