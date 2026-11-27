"""InventoryTracker descriptions, labels and Invsee command help in Chinese."""
from pathlib import Path
import argparse, json
from display_locale_schema import put_module
HERE = Path(__file__).resolve().parent
TEXTS = {
    'description': (
        'Track items and armor observed on other players; use invsee to view their recorded inventory.',
        '记录其他玩家手持和穿戴过的物品，使用 invsee 命令查看已观察到的背包。',
        '記錄其他玩家手持與穿戴過的物品，使用 invsee 指令查看已觀察到的背包。'),
    'savePlayers.description': (
        'Retain player objects outside render distance. Changing this setting clears saved players, while recorded item history remains.',
        '保留离开渲染距离的玩家对象。更改此选项会清除保存的玩家，但保留已记录的物品历史。',
        '保留離開繪製距離的玩家物件。變更此選項會清除已儲存的玩家，但保留物品觀察紀錄。'),
}


def update(folder):
    for locale, index in [('zh_cn', 1), ('zh_tw', 2)]:
        path = folder / f'display_{locale}.json'
        data = json.loads(path.read_text(encoding='utf8'))
        put_module(data, 'liquidbounce.module.inventoryTracker', 'InventoryTracker',
                   {'': ('背包记录', '背包紀錄')[index - 1], **{k: v[index] for k, v in TEXTS.items()}},
                   {k: v[0] for k, v in TEXTS.items()})
        data['labels']['SavePlayers'] = ('保留玩家', '保留玩家')[index - 1]
        data['descriptions']['liquidbounce.command.invsee.description'] = (
            '查看在线玩家的已知背包，玩家离开渲染距离后可使用“保留玩家”继续查看。',
            '查看線上玩家的已知背包，玩家離開繪製距離後可透過「保留玩家」繼續查看。')[index - 1]
        if 'englishDescriptions' in data:
            data['englishDescriptions']['liquidbounce.command.invsee.description'] = (
                'View an online player’s known inventory; SavePlayers retains players outside render distance.')
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder', type=Path, default=HERE.parent / 'liquidbounce-interop/resources/localization')
    update(parser.parse_args().folder)
