#!/usr/bin/env python3
"""Original GlobalSettingsInventory values and manager response/locking help."""
from pathlib import Path
import json
ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'native/liquidbounce-interop/resources/localization'
TEXTS = {
    'responseTime': (
        'Allow this many milliseconds for the server to open a container, restore a clicked item, or undo that restore. Inventory clicks wait while a reply may still change their result. Default 400 ms; the original slider range is 50–2000 ms.',
        '等待服务器打开容器、放回被点击的物品或撤销该次放回的毫秒数。服务器回复仍可能改变物品栏状态时，暂停相关点击。默认 400 毫秒；原始滑块范围为 50–2000 毫秒。',
        '等待伺服器開啟容器、放回被點擊的物品或撤銷該次放回的毫秒數。伺服器回覆仍可能改變物品欄狀態時，暫停相關點擊。預設 400 毫秒；原始滑桿範圍為 50–2000 毫秒。'),
    'lockAfterRejections': (
        'After the server restores the same clicked item this many times, leave it alone until the world changes. Default 3; the original slider range is 1–10.',
        '服务器反复放回同一被点击物品达到此次数后，不再移动该物品，直到切换世界。默认 3 次；原始滑块范围为 1–10 次。',
        '伺服器反覆放回同一被點擊物品達到此次數後，不再移動該物品，直到切換世界。預設 3 次；原始滑桿範圍為 1–10 次。'),
}
for locale, index, labels in [
    ('zh_cn', 1, {'ResponseTime': '响应等待时间', 'LockAfterRejections': '拒绝锁定次数'}),
    ('zh_tw', 2, {'ResponseTime': '回應等待時間', 'LockAfterRejections': '拒絕鎖定次數'}),
]:
    path = FOLDER / f'display_{locale}.json'
    data = json.loads(path.read_text(encoding='utf-8'))
    data['labels'].update(labels)
    for name, texts in TEXTS.items():
        key = f'liquidbounce.settings.inventory.{name}.description'
        data['descriptions'][key] = texts[index]
        if locale == 'zh_cn':
            data['englishDescriptions'][key] = texts[0]
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'{locale}: {len(data["descriptions"])} owned descriptions')
