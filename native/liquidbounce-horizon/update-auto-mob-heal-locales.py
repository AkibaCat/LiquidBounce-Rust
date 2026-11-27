#!/usr/bin/env python3
"""Populate AutoMobHeal's complete native settings tree in both Chinese locales."""
from pathlib import Path
import argparse
import json
from display_locale_schema import put_module

HERE = Path(__file__).resolve().parent
PREFIX = "liquidbounce.module.autoMobHeal"
LABELS = {
    "AutoMobHeal": ("自动治疗生物", "自動治療生物"),
    "Range": ("交互范围", "互動範圍"),
    "Delay": ("治疗间隔", "治療間隔"),
    "SlotResetDelay": ("槽位恢复延迟", "欄位復原延遲"),
    "SwingMode": ("挥手显示", "揮手顯示"),
    "DoNotHide": ("显示挥手", "顯示揮手"),
    "HideForClient": ("本地隐藏挥手", "本機隱藏揮手"),
    "Enabled": ("启用", "啟用"),
    "HealthThreshold": ("生命值阈值", "生命值門檻"),
    "NoWaste": ("避免浪费食物", "避免浪費食物"),
    "Armor": ("狼铠管理", "狼鎧管理"),
    "Equip": ("自动装备狼铠", "自動裝備狼鎧"),
    "RepairThreshold": ("修复耐久阈值", "修復耐久門檻"),
    "OnlyWhenSitting": ("仅修复坐下的狼", "僅修復坐下的狼"),
    "AllowGoldenCarrot": ("允许金胡萝卜", "允許金胡蘿蔔"),
    "AllowGoldenApple": ("允许金苹果", "允許金蘋果"),
    "AllowEnchantedGoldenApple": ("允许附魔金苹果", "允許附魔金蘋果"),
    "AllowBucketFood": ("允许桶装鱼", "允許桶裝魚"),
}
KINDS = [
    ("IronGolem", "iron golems", "铁傀儡", "鐵魔像"),
    ("Wolf", "wolves", "狼", "狼"),
    ("Cat", "cats", "猫", "貓"),
    ("HorseFamily", "horses, donkeys and mules", "马、驴和骡", "馬、驢與騾"),
    ("ZombieHorse", "zombie horses", "僵尸马", "殭屍馬"),
    ("Llama", "llamas", "羊驼", "駱馬"),
    ("Camel", "camels", "骆驼", "駱駝"),
    ("CamelHusk", "camel husks", "尸壳骆驼", "屍殼駱駝"),
    ("Nautilus", "nautiluses", "鹦鹉螺", "鸚鵡螺"),
]
for tag, _, cn, tw in KINDS:
    LABELS[tag] = (cn, tw)
TEXTS = {}
SCOPED = {}
def camel(path):
    return ".".join(part[:1].lower() + part[1:] for part in path.split("."))
def setting(path, en, cn, tw):
    key = camel(path)
    TEXTS[key + ".description" if key else "description"] = (en, cn, tw)
    if path:
        SCOPED[key] = LABELS[path.rsplit(".", 1)[-1]]

setting("", "Automatically heal supported mobs and optionally equip or repair owned wolves' armor.",
        "自动治疗受支持的生物，也可为自己的狼装备或修复狼铠。",
        "自動治療受支援的生物，也可為自己的狼裝備或修復狼鎧。")
setting("Range", "Maximum eye-to-mob distance, also limited by the player's actual entity interaction range.",
        "眼睛到生物碰撞箱的最大距离，同时受玩家实际实体交互范围限制。",
        "眼睛到生物碰撞箱的最大距離，同時受玩家實際實體互動範圍限制。")
setting("Delay", "Random ticks to wait after an interaction is consumed before healing again.",
        "交互成功消耗操作后，随机等待这些游戏刻再进行下一次治疗。",
        "互動成功消耗操作後，隨機等待這些遊戲刻再進行下一次治療。")
setting("SlotResetDelay", "Ticks before restoring the original hotbar slot after a silent selection.",
        "静默切换快捷栏后，经过这些游戏刻恢复原槽位。",
        "靜默切換快捷列後，經過這些遊戲刻復原原欄位。")
setting("SwingMode", "Choose whether interaction swings are visible locally; the server still receives the normal swing.",
        "选择是否在本地显示交互挥手；服务器仍收到正常挥手。",
        "選擇是否在本機顯示互動揮手；伺服器仍收到正常揮手。")
for option, en, cn, tw in [
    ("DoNotHide", "Show the normal interaction swing on the client.", "在客户端显示正常交互挥手。", "在用戶端顯示正常互動揮手。"),
    ("HideForClient", "Hide the interaction swing only on the client.", "仅在本地隐藏交互挥手。", "僅在本機隱藏互動揮手。"),
]:
    SCOPED["swingMode." + option] = LABELS[option]
    TEXTS["swingMode." + camel(option) + ".description"] = (en, cn, tw)
for tag, en, cn, tw in KINDS:
    setting(tag, f"Configure automatic healing for {en}.", f"配置{cn}的自动治疗。", f"設定{tw}的自動治療。")
    setting(tag + ".Enabled", f"Enable automatic healing for {en}.", f"启用{cn}的自动治疗。", f"啟用{tw}的自動治療。")
    setting(tag + ".HealthThreshold", "Heal while health is at or below this percentage of maximum health.",
            "生命值不高于最大生命值的此百分比时治疗。", "生命值不高於最大生命值的此百分比時治療。")
    if tag != "IronGolem":
        setting(tag + ".NoWaste", "Reject foods whose healing exceeds the mob's missing health.",
                "不使用恢复量大于生物缺失生命值的食物。", "不使用恢復量大於生物缺失生命值的食物。")
setting("Wolf.Armor", "Equip or repair body armor on wolves owned by the local player.",
        "为本地玩家拥有的狼装备或修复狼铠。", "為本機玩家擁有的狼裝備或修復狼鎧。")
setting("Wolf.Armor.Enabled", "Enable armor equipment and repair checks for owned wolves.",
        "启用对自己的狼的狼铠装备与修复检查。", "啟用對自己的狼的狼鎧裝備與修復檢查。")
setting("Wolf.Armor.Equip", "Equip wolf armor from the hotbar or offhand on an owned adult wolf without body armor.",
        "使用快捷栏或副手中的狼铠，为尚未穿戴狼铠的自己的成年狼装备。",
        "使用快捷列或副手中的狼鎧，為尚未穿戴狼鎧的自己的成年狼裝備。")
setting("Wolf.Armor.RepairThreshold", "Repair armor at or below this percentage of remaining durability using an item accepted by its repair component.",
        "狼铠剩余耐久不高于此百分比时，使用其修复组件允许的材料修复。",
        "狼鎧剩餘耐久不高於此百分比時，使用其修復元件允許的材料修復。")
setting("Wolf.Armor.OnlyWhenSitting", "Only repair wolf armor while the wolf is sitting; this does not restrict feeding or equipping armor.",
        "仅在狼坐下时修复狼铠；不限制喂食或装备狼铠。",
        "僅在狼坐下時修復狼鎧；不限制餵食或裝備狼鎧。")
for item, en, cn, tw in [
    ("GoldenCarrot", "golden carrots", "金胡萝卜", "金胡蘿蔔"),
    ("GoldenApple", "golden apples", "金苹果", "金蘋果"),
    ("EnchantedGoldenApple", "enchanted golden apples", "附魔金苹果", "附魔金蘋果"),
]:
    setting("HorseFamily.Allow" + item, f"Allow {en} when choosing food for horses, donkeys and mules.",
            f"为马、驴和骡选择治疗食物时允许使用{cn}。", f"為馬、驢與騾選擇治療食物時允許使用{tw}。")
setting("Nautilus.AllowBucketFood", "Allow fish buckets as nautilus food, preferring suitable fish without buckets.",
        "允许用桶装鱼喂食鹦鹉螺，并优先选择合适的非桶装鱼。",
        "允許用桶裝魚餵食鸚鵡螺，並優先選擇合適的非桶裝魚。")

def update(folder: Path):
    for locale, index in (("zh_cn", 1), ("zh_tw", 2)):
        path = folder / f"display_{locale}.json"
        data = json.loads(path.read_text(encoding="utf8"))
        entries = {"": LABELS["AutoMobHeal"][index - 1]}
        entries.update({key: value[index] for key, value in TEXTS.items()})
        put_module(data, PREFIX, "AutoMobHeal", entries, {key: value[0] for key, value in TEXTS.items()})
        labels = data.setdefault("labels", {})
        for key, value in LABELS.items():
            labels[key] = value[index - 1]
        for key, value in SCOPED.items():
            labels[PREFIX + "." + key] = value[index - 1]
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf8")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--folder", type=Path, default=HERE.parent / "liquidbounce-interop/resources/localization")
    update(parser.parse_args().folder)
