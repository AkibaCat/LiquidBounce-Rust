"""Complete Nametags Simplified/Traditional Chinese settings and choice labels."""
from pathlib import Path
import argparse, json
from display_locale_schema import put_module

HERE = Path(__file__).resolve().parent

TEXTS = {
    "description": ("Show names, status text and equipment above living entities.", "在生物上方显示名称、状态文字和装备。", "在生物上方顯示名稱、狀態文字與裝備。"),
    "text.description": ("Choose the information displayed in the name tag text.", "选择名牌文字中显示的信息。", "選擇名稱標籤文字中顯示的資訊。"),
    "text.parts.description": ("Ordered text parts shown in each name tag.", "按顺序选择名牌中显示的文字部分。", "依順序選擇名稱標籤中顯示的文字部分。"),
    "equipment.description": ("Choose equipment slots and item details drawn below the name.", "选择名称下方显示的装备槽位和物品信息。", "選擇名稱下方顯示的裝備欄位與物品資訊。"),
    "equipment.slots.description": ("Ordered equipment slots shown below each name tag.", "按顺序选择名称下方显示的装备槽位。", "依順序選擇名稱下方顯示的裝備欄位。"),
    "equipment.skipEmptySlot.description": ("Hide equipment slots that contain no item.", "隐藏没有物品的装备槽位。", "隱藏沒有物品的裝備欄位。"),
    "equipment.showInfo.description": ("Draw item counts, durability bars and enchantment labels.", "显示物品数量、耐久条和附魔标签。", "顯示物品數量、耐久條與附魔標籤。"),
    "equipment.highlightItemInUse.description": ("Highlight the exact item currently being used.", "高亮当前正在使用的那个物品。", "高亮目前正在使用的那個物品。"),
    "equipment.highlightItemInUse.enabled.description": ("Enable the item-in-use highlight.", "启用使用中物品高亮。", "啟用使用中物品高亮。"),
    "equipment.highlightItemInUse.fillColor.description": ("Fill color for the item-in-use highlight.", "使用中物品高亮的填充颜色。", "使用中物品高亮的填滿色彩。"),
    "equipment.highlightItemInUse.outlineColor.description": ("Outline color for the item-in-use highlight.", "使用中物品高亮的轮廓颜色。", "使用中物品高亮的輪廓色彩。"),
    "equipment.enchantment.description": ("Show abbreviated enchantments above each item.", "在每件物品上方显示缩写后的附魔。", "在每件物品上方顯示縮寫後的附魔。"),
    "equipment.enchantment.enabled.description": ("Enable enchantment labels.", "启用附魔标签。", "啟用附魔標籤。"),
    "equipment.enchantment.scale.description": ("Scale of enchantment labels.", "附魔标签的缩放比例。", "附魔標籤的縮放比例。"),
    "equipment.enchantment.maxCountPerItem.description": ("Maximum number of enchantment labels per item.", "每件物品最多显示的附魔标签数量。", "每件物品最多顯示的附魔標籤數量。"),
    "equipment.enchantment.backgroundRadius.description": ("Corner radius of enchantment label backgrounds.", "附魔标签背景的圆角半径。", "附魔標籤背景的圓角半徑。"),
    "equipment.enchantment.enchantments.description": ("Enchantment identifiers used by the whitelist or blacklist.", "白名单或黑名单使用的附魔标识符。", "白名單或黑名單使用的附魔識別字。"),
    "equipment.enchantment.filter.description": ("Choose whether the listed enchantments are shown or hidden.", "选择显示列表中的附魔或隐藏列表中的附魔。", "選擇顯示清單中的附魔或隱藏清單中的附魔。"),
    "borderWidth.description": ("Width of the name tag outline.", "名牌轮廓的宽度。", "名稱標籤輪廓的寬度。"),
    "backgroundRadius.description": ("Corner radius of the name tag background.", "名牌背景的圆角半径。", "名稱標籤背景的圓角半徑。"),
    "scale.description": ("Set the name tag scale at each camera distance.", "通过曲线设置不同相机距离下名牌的缩放。", "透過曲線設定不同攝影機距離下名稱標籤的縮放。"),
}

LABELS = {
    "Distance": ("距离", "距離"), "Ping": ("延迟", "延遲"), "Name": ("名称", "名稱"),
    "Health": ("生命值", "生命值"), "GameMode": ("游戏模式", "遊戲模式"), "BotMark": ("机器人标记", "機器人標記"),
    "TotemPops": ("图腾触发次数", "圖騰觸發次數"), "Mainhand": ("主手", "主手"), "Offhand": ("副手", "副手"),
    "Feet": ("脚部", "腳部"), "Legs": ("腿部", "腿部"), "Head": ("头部", "頭部"), "Body": ("躯干", "軀幹"),
    "Saddle": ("鞍", "鞍"), "SkipEmptySlot": ("跳过空槽位", "跳過空欄位"), "ShowInfo": ("显示信息", "顯示資訊"),
    "Enchantment": ("附魔", "附魔"), "HighlightItemInUse": ("高亮使用中的物品", "高亮使用中的物品"),
    "Enabled": ("启用", "啟用"), "Scale": ("缩放", "縮放"), "MaxCountPerItem": ("每种物品最大数量", "每種物品最大數量"),
    "BackgroundRadius": ("背景圆角", "背景圓角"), "Filter": ("筛选器", "篩選器"), "Enchantments": ("附魔列表", "附魔清單"),
    "Whitelist": ("白名单", "白名單"), "Blacklist": ("黑名单", "黑名單"), "FillColor": ("填充颜色", "填滿色彩"),
    "OutlineColor": ("轮廓颜色", "輪廓色彩"), "BorderWidth": ("边框宽度", "邊框寬度"),
}

SCOPED = {
    "liquidbounce.module.nametags.text.parts": {
        "Distance": ("距离", "距離"), "Ping": ("延迟", "延遲"), "Name": ("名称", "名稱"),
        "Health": ("生命值", "生命值"), "GameMode": ("游戏模式", "遊戲模式"), "BotMark": ("机器人标记", "機器人標記"), "TotemPops": ("图腾触发次数", "圖騰觸發次數"),
    },
    "liquidbounce.module.nametags.equipment.slots": {
        "Mainhand": ("主手", "主手"), "Offhand": ("副手", "副手"), "Feet": ("脚部", "腳部"), "Legs": ("腿部", "腿部"),
        "Chest": ("胸部", "胸部"), "Head": ("头部", "頭部"), "Body": ("躯干", "軀幹"), "Saddle": ("鞍", "鞍"),
    },
}


def update(folder: Path):
    for code, index in [("zh_cn", 1), ("zh_tw", 2)]:
        path = folder / f"display_{code}.json"
        data = json.loads(path.read_text(encoding="utf8"))
        english = {key: value[0] for key, value in TEXTS.items()}
        translated = {key: value[index] for key, value in TEXTS.items()}
        put_module(data, "liquidbounce.module.nametags", "Nametags", {"": ("名称标签", "名稱標籤")[index - 1], **translated}, english)
        for key, values in LABELS.items():
            data.setdefault("labels", {})[key] = values[index - 1]
        for prefix, values in SCOPED.items():
            for key, pair in values.items():
                data.setdefault("labels", {})[prefix + "." + key] = pair[index - 1]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--folder", type=Path, default=HERE.parent / "liquidbounce-interop/resources/localization")
    update(parser.parse_args().folder)

