#!/usr/bin/env python3
"""Anchor, AvoidHazards and Freeze display vocabulary; preserve wire names."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
MODULES = {
    'Anchor': ('洞穴锚定', '洞穴錨定'),
    'AvoidHazards': ('危险规避', '危險規避'),
    'Freeze': ('冻结', '凍結'),
}
LABELS = {
    'MaxDistance': ('最大距离', '最大距離'),
    'HorizontalSpeed': ('水平速度', '水平速度'),
    'VerticalSpeed': ('垂直速度', '垂直速度'),
    'Shape': ('碰撞形状', '碰撞形狀'),
    'Input': ('移动输入', '移動輸入'),
    'Avoid': ('规避目标', '規避目標'),
    'Cacti': ('仙人掌', '仙人掌'),
    'BerryBush': ('甜浆果丛', '甜漿果叢'),
    'Fire': ('火焰', '火焰'),
    'Cobwebs': ('蜘蛛网', '蜘蛛網'),
    'Ladders': ('梯子', '梯子'),
    'PressurePlates': ('压力板', '壓力板'),
    'MagmaBlocks': ('岩浆块', '岩漿塊'),
    'Lava': ('熔岩', '熔岩'),
    'WitherRose': ('凋灵玫瑰', '凋零玫瑰'),
    'PowderSnow': ('细雪', '粉雪'),
    'Queue': ('排队暂存', '排隊暫存'),
    'Cancel': ('取消', '取消'),
    'Stationary': ('原地冻结', '原地凍結'),
    'TickMovement': ('间隔移动', '間隔移動'),
    'Origin': ('传输方向', '傳輸方向'),
    'Incoming': ('接收', '接收'),
    'Outgoing': ('发送', '傳送'),
    'CancelC0B': ('取消延迟回复包', '取消延遲回覆封包'),
    'DisableOn': ('自动停用条件', '自動停用條件'),
    'Notification': ('通知', '通知'),
    'Flag': ('服务器位置修正', '伺服器位置修正'),
    'OnGround': ('接触地面', '接觸地面'),
    'OnMovementInput': ('按下移动键', '按下移動鍵'),
    'InLiquid': ('进入液体', '進入液體'),
    'Void': ('进入虚空', '進入虛空'),
    'OnUseItem': ('使用物品', '使用物品'),
}
# Description keys match the settings tree and upstream translation convention.
TEXTS = {
    'anchor': ('Pulls you into safe holes for crystal PvP.', '将你拉入适合水晶战斗的安全洞穴。', '將你拉入適合水晶戰鬥的安全洞穴。'),
    'anchor.maxDistance': ('Maximum distance to a safe hole, from 0.1 to 6 blocks. Default 4.', '搜索安全洞穴的最大距离为 0.1–6 格，默认 4 格。', '搜尋安全洞穴的最大距離為 0.1–6 格，預設 4 格。'),
    'anchor.horizontalSpeed': ('Maximum horizontal movement applied toward the hole. Default 0.3.', '朝洞穴移动时施加的最大水平速度，默认 0.3。', '朝洞穴移動時施加的最大水平速度，預設 0.3。'),
    'anchor.verticalSpeed': ('Maximum downward movement applied toward the hole. Default 0.1.', '朝洞穴移动时施加的最大下降速度，默认 0.1。', '朝洞穴移動時施加的最大下降速度，預設 0.1。'),
    'avoidHazards': ('Prevents you from walking into blocks that might affect you negatively.', '避免走入可能伤害或妨碍移动的方块。', '避免走入可能傷害或妨礙移動的方塊。'),
    'avoidHazards.mode': ('Shape adds collision to selected hazards. Input predicts movement and redirects movement keys.', '碰撞形状为选中的危险方块添加碰撞；移动输入通过预测移动来调整方向键。', '碰撞形狀為選中的危險方塊加入碰撞；移動輸入透過預測移動來調整方向鍵。'),
    'avoidHazards.avoid': ('Hazards to avoid. All ten types are enabled by default.', '选择要规避的危险方块，默认启用全部十种。', '選擇要規避的危險方塊，預設啟用全部十種。'),
    'freeze': ('Allows you to freeze yourself in your current position without the server knowing.', '在当前位置冻结自身移动，并按所选模式处理网络数据包。', '在目前位置凍結自身移動，並按所選模式處理網路封包。'),
    'freeze.mode': ('Queue buffers packets, Cancel drops packets, Stationary sends interaction rotations, and TickMovement releases movement at intervals.', '排队暂存会缓存数据包；取消会丢弃数据包；原地冻结会发送交互所需的视角；间隔移动会定期放行移动。', '排隊暫存會快取封包；取消會捨棄封包；原地凍結會傳送互動所需的視角；間隔移動會定期放行移動。'),
    'freeze.mode.queue.origin': ('Packet directions to buffer until Freeze is disabled. Outgoing by default.', '选择冻结期间缓存的数据包方向，停用后释放，默认仅发送。', '選擇凍結期間快取的封包方向，停用後釋放，預設僅傳送。'),
    'freeze.mode.cancel.origin': ('Packet directions to drop while frozen. Outgoing by default.', '选择冻结期间丢弃的数据包方向，默认仅发送。', '選擇凍結期間捨棄的封包方向，預設僅傳送。'),
    'freeze.mode.stationary.cancelC0B': ('Cancel the pong reply while frozen. Enabled by default.', '冻结时取消延迟检测回复包，默认启用。', '凍結時取消延遲偵測回覆封包，預設啟用。'),
    'freeze.mode.tickMovement.interval': ('Ticks between released movement updates. Default 20 ticks.', '两次放行移动更新之间的游戏刻数，默认 20 刻。', '兩次放行移動更新之間的遊戲刻數，預設 20 刻。'),
    'freeze.disableOn': ('Disable Freeze when selected conditions occur. Server correction is selected by default.', '在选定条件发生时停用冻结，默认选择服务器位置修正。', '在選定條件發生時停用凍結，預設選擇伺服器位置修正。'),
    'freeze.notification': ('Show a notification when Freeze is disabled by a selected condition.', '冻结因选定条件自动停用时显示通知。', '凍結因選定條件自動停用時顯示通知。'),
}

def main():
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = FOLDER / f'display_{locale}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        data['moduleNames'].update({k: v[index-1] for k, v in MODULES.items()})
        data['labels'].update({k: v[index-1] for k, v in LABELS.items()})
        for name, texts in TEXTS.items():
            key = f'liquidbounce.module.{name}.description'
            data['descriptions'][key] = texts[index]
            if locale == 'zh_cn':
                data['englishDescriptions'][key] = texts[0]
        key = 'liquidbounce.module.freeze.messages.disabled'
        data['descriptions'][key] = ('因%s而停用模块。', '因%s而停用模組。')[index-1]
        if locale == 'zh_cn':
            data['englishDescriptions'][key] = 'Module disabled due to %s.'
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2)+'\n', encoding='utf8')
        print(locale, len(data['descriptions']))

if __name__ == '__main__':
    main()
