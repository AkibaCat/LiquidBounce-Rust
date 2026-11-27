"""Populate every Scaffold/Clutch schema path with Simplified/Traditional Chinese UI text."""
from pathlib import Path
import argparse, json, re

HERE = Path(__file__).resolve().parent
SCHEMA = HERE / "engine/src/config/scaffold_schema.json"

TERMS = {
    "Delay": ("延迟", "延遲"), "MinDist": ("最小距离", "最小距離"), "Timer": ("计时器", "計時器"),
    "BlockItemSelection": ("方块物品选择", "方塊物品選擇"), "Disallowed": ("禁止方块", "禁止方塊"), "Unfavorable": ("不优方块", "不利方塊"),
    "AutoBlock": ("自动方块", "自動方塊"), "Always": ("始终启用", "永遠啟用"), "SlotResetDelay": ("槽位重置延迟", "欄位重設延遲"),
    "DoNotUseBelowCount": ("低于数量时停用", "低於數量時停用"), "Prediction": ("预测", "預測"), "BootstrapBackoff": ("启动退避", "啟動退避"),
    "PredictionCutoffDistance": ("预测截止距离", "預測截止距離"), "WarmupPlacements": ("预热放置数", "預熱放置數"), "Technique": ("技术", "技術"),
    "Normal": ("普通", "普通"), "RotationMode": ("旋转模式", "旋轉模式"), "RequiresSight": ("需要视线", "需要視線"),
    "Eagle": ("鹰式搭路", "鷹式搭路"), "BlocksToEagle": ("触发鹰式搭路的方块数", "觸發鷹式搭路的方塊數"), "EdgeDistance": ("边缘距离", "邊緣距離"),
    "OnlyOnGround": ("仅在地面", "僅在地面"), "Telly": ("Telly", "Telly"), "ResetMode": ("重置模式", "重設模式"),
    "Straight": ("直线时间", "直線時間"), "Jump": ("跳跃", "跳躍"), "AimOnTower": ("搭塔时瞄准", "搭塔時瞄準"),
    "Down": ("向下搭路", "向下搭路"), "StabilizeMovement": ("稳定移动", "穩定移動"), "Ceiling": ("天花板", "天花板"),
    "HeadHitter": ("顶头", "頂頭"), "JumpDelay": ("跳跃延迟", "跳躍延遲"), "Expand": ("扩展", "擴展"), "Length": ("长度", "長度"),
    "GodBridge": ("GodBridge", "GodBridge"), "Backwards": ("向后", "向後"), "StopInput": ("停止输入", "停止輸入"), "Modes": ("模式", "模式"), "ForceSneakBelowCount": ("低于数量时强制潜行", "低於數量時強制潛行"),
    "SneakTime": ("潜行时间", "潛行時間"), "Breezily": ("Breezily", "Breezily"), "SameY": ("保持同一高度", "保持同一高度"),
    "Off": ("关闭", "關閉"), "On": ("开启", "開啟"), "JumpKey": ("跳跃键", "跳躍鍵"), "Falling": ("下落", "下落"),
    "Hypixel": ("Hypixel", "Hypixel"), "Tower": ("搭塔", "搭塔"), "None": ("无", "無"), "Motion": ("运动", "運動"),
    "TriggerHeight": ("触发高度", "觸發高度"), "Slow": ("减速", "減速"), "Pulldown": ("下拉", "下拉"), "Trigger": ("触发阈值", "觸發閾值"),
    "Karhu-2024-05-10": ("Karhu（2024-05-10）", "Karhu（2024-05-10）"), "Vulcan-2.8.8": ("Vulcan（2.8.8）", "Vulcan（2.8.8）"),
    "Hypixel-2026-07-12": ("Hypixel（2026-07-12）", "Hypixel（2026-07-12）"), "SafeWalk": ("安全行走", "安全行走"),
    "Safe": ("安全", "安全"), "OnEdge": ("边缘", "邊緣"), "Distance": ("距离", "距離"), "Keep": ("保持时间", "保持時間"),
    "Mode": ("模式", "模式"), "Stop": ("停止", "停止"), "Invert": ("反向", "反向"), "Center": ("居中", "置中"), "Sneak": ("潜行", "潛行"),
    "Rotations": ("旋转", "旋轉"), "ConsiderInventory": ("考虑物品栏", "考慮物品欄"), "RotationTiming": ("旋转时机", "旋轉時機"),
    "OnTick": ("每 tick", "每 tick"), "OnTickSnap": ("每 tick 瞬转", "每 tick 瞬轉"), "Swing": ("挥手", "揮手"),
    "DoNotHide": ("不隐藏", "不隱藏"), "HideForClient": ("仅对客户端隐藏", "僅對用戶端隱藏"), "SprintControl": ("疾跑控制", "疾跑控制"),
    "Client": ("客户端", "用戶端"), "Server": ("服务器", "伺服器"), "DoNotChange": ("不改变", "不變更"),
    "ForceSprint": ("强制疾跑", "強制疾跑"), "ForceNoSprint": ("强制停止疾跑", "強制停止疾跑"), "NoSprintOnPlace": ("放置时停止疾跑", "放置時停止疾跑"),
    "NoSprintOnGround": ("落地时停止疾跑", "落地時停止疾跑"), "SimulatePlacementAttempts": ("模拟放置尝试", "模擬放置嘗試"),
    "Clicker": ("点击器", "點擊器"), "Human": ("拟人", "擬人"), "Constant": ("恒定", "恆定"), "CPS": ("每秒点击数", "每秒點擊數"),
    "MaxPerTick": ("每 tick 最大次数", "每 tick 最大次數"), "FailedAttemptsOnly": ("仅失败尝试", "僅失敗嘗試"), "Acceleration": ("加速", "加速"),
    "SpeedMultiplier": ("速度倍率", "速度倍率"), "Strafe": ("侧移", "側移"), "Speed": ("速度", "速度"), "StrafeOnJump": ("跳跃侧移", "跳躍側移"),
    "StraightSpeed": ("直线速度", "直線速度"), "DiagonalSpeed": ("斜向速度", "斜向速度"), "SpeedLimiter": ("速度限制", "速度限制"),
    "SpeedLimit": ("速度上限", "速度上限"), "Blink": ("闪烁", "閃爍"), "Time": ("时间", "時間"), "FlushOn": ("刷新条件", "刷新條件"),
    "Place": ("放置", "放置"), "Towering": ("搭塔", "搭塔"), "Sneaking": ("潜行中", "潛行中"), "NotSneaking": ("未潜行", "未潛行"),
    "InAir": ("空中", "空中"), "AutoSpeed": ("自动速度", "自動速度"), "Ledge": ("边缘检测", "邊緣偵測"), "Render": ("渲染", "渲染"),
    "Enabled": ("启用", "啟用"), "Blocks": ("方块", "方塊"), "OnGround": ("在地面", "在地面"), "Reverse": ("反向", "反向"),
    "Reset": ("重置", "重設"), "Random": ("随机", "隨機"), "Stabilized": ("稳定", "穩定"), "NearestRotation": ("最近旋转", "最近旋轉"),
    "ReverseYaw": ("反向偏航", "反向偏航"), "DiagonalYaw": ("对角偏航", "對角偏航"), "AngleYaw": ("角度偏航", "角度偏航"), "EdgePoint": ("边缘点", "邊緣點"),
}


def camel(path):
    return ".".join(p[:1].lower() + p[1:] for p in path.split("."))


def title(name):
    if name in TERMS:
        return TERMS[name]
    text = re.sub(r"(?<!^)([A-Z])", r" \1", name).replace("_", " ")
    return (text, text)


def walk(nodes, prefix=""):
    for node in nodes:
        name = node.get("name")
        if not name:
            continue
        path = f"{prefix}.{name}" if prefix else name
        yield path, node
        for key in ("children", "modes"):
            if isinstance(node.get(key), list):
                yield from walk(node[key], path)


def update(folder: Path):
    schema = json.loads(SCHEMA.read_text(encoding="utf8"))
    for code, index in (("zh_cn", 0), ("zh_tw", 1)):
        path = folder / f"display_{code}.json"
        data = json.loads(path.read_text(encoding="utf8"))
        labels = data.setdefault("labels", {})
        descriptions = data.setdefault("descriptions", {})
        module_key = "liquidbounce.module.scaffold"
        data.setdefault("moduleNames", {})["Scaffold"] = ("自动搭路", "自動搭路")[index]
        data["moduleNames"]["Clutch"] = ("落地自救", "落地自救")[index]
        descriptions[module_key + ".description"] = ("自动在脚下放置方块。", "自動在腳下放置方塊。")[index]
        descriptions["liquidbounce.module.clutch.description"] = ("在坠落时放置方块以避免摔落伤害。", "在墜落時放置方塊以避免摔落傷害。")[index]
        for raw, node in walk(schema):
            key = module_key + "." + camel(raw)
            name = node.get("name", raw.rsplit(".", 1)[-1])
            tr = title(name)[index]
            labels[name] = tr
            labels[key] = tr
            descriptions[key + ".description"] = f"{tr}设置。" if index == 0 else f"{tr}設定。"
            for option in node.get("options", []):
                otr = title(option)[index]
                labels[option] = otr
                labels[key + "." + option] = otr
                descriptions[key + "." + option[:1].lower() + option[1:] + ".description"] = f"{otr}模式。" if index == 0 else f"{otr}模式。"
            for mode in node.get("modes", []):
                option = mode if isinstance(mode, str) else mode.get("name")
                if option:
                    otr = title(option)[index]
                    labels[option] = otr
                    labels[key + "." + option] = otr
                    descriptions[key + "." + option[:1].lower() + option[1:] + ".description"] = f"{otr}模式。" if index == 0 else f"{otr}模式。"
        # The shared providers use these keys and are part of the same module tree.
        for path_key, zh, tw in [
            ("rotations", "配置旋转行为。", "設定旋轉行為。"),
            ("render", "配置放置预览渲染。", "設定放置預覽渲染。"),
            ("simulatePlacementAttempts.clicker", "配置模拟点击行为。", "設定模擬點擊行為。"),
        ]:
            descriptions[f"{module_key}.{path_key}.description"] = (zh, tw)[index]
        if "englishDescriptions" in data:
            data["englishDescriptions"].update({
                module_key + "." + camel(raw) + ".description": f"{node.get('name', raw.rsplit('.', 1)[-1])} setting."
                for raw, node in walk(schema)
            })
            data["englishDescriptions"][module_key + ".description"] = "Automatically place blocks under the player."
            data["englishDescriptions"]["liquidbounce.module.clutch.description"] = "Place blocks while falling to prevent fall damage."
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--folder", type=Path, default=HERE.parent / "liquidbounce-interop/resources/localization")
    update(parser.parse_args().folder)

