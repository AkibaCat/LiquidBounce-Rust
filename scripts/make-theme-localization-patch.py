#!/usr/bin/env python3
"""Reproduce the display-only Chinese theme patch against the pinned source."""
from __future__ import annotations

import difflib
import json
from pathlib import Path
import posixpath
import re

PROJECT = Path(__file__).resolve().parent.parent
SOURCE = PROJECT / "upstream/LiquidBounce/src-theme"
PATCH = PROJECT / "patches/liquidbounce-theme-chinese.patch"
DISPLAY = PROJECT / "native/liquidbounce-interop/resources/localization/display_zh_cn.json"


def replace_once(text: str, before: str, after: str) -> str:
    if text.count(before) != 1:
        raise ValueError(f"Expected one pinned source anchor, got {text.count(before)}: {before[:90]}")
    return text.replace(before, after)


def add_import(text: str, relative: str, names: set[str]) -> str:
    if not names:
        return text
    target = posixpath.relpath("src/integration/localization", posixpath.dirname(relative))
    if not target.startswith("."):
        target = "./" + target
    declaration = '    import {' + ", ".join(sorted(names)) + '} from "' + target + '";\n'
    match = re.search(r"<script(?:\s+[^>]*)?>\n", text)
    if match:
        return text[:match.end()] + declaration + text[match.end():]
    return '<script lang="ts">\n' + declaration + "</script>\n\n" + text


def patch_source(relative: str, original: str, labels: dict[str, str]) -> str:
    text = original
    imports: set[str] = set()
    if relative.endswith(".svelte"):
        if relative in {
            "src/routes/menu/singleplayer/Singleplayer.svelte",
            "src/routes/menu/multiplayer/Multiplayer.svelte",
        }:
            # Use the theme's own icon when the saved entry has no thumbnail.
            # Horizon does not expose Minecraft's generic resource REST route.
            text = replace_once(text,
                "`${REST_BASE}/api/v1/client/resource?id=minecraft:textures/misc/unknown_server.png`",
                '"/img/menu/icon-server.svg"')
        if relative == "src/App.svelte":
            # ClickGUI components read localStorage synchronously and can queue
            # a whole-map write during mount. Construct them only after the
            # native persistent snapshot has arrived, including on Ctrl+R.
            text = replace_once(text, "    const SURFACE_TINT_MIX = 18;",
                "    let persistentReady = false;\n\n    const SURFACE_TINT_MIX = 18;")
            text = replace_once(text, "        await insertPersistentData();",
                "        await insertPersistentData();\n        persistentReady = true;")
            text = replace_once(text, "<main>\n", "<main>\n    {#if persistentReady}\n")
            text = replace_once(text, "</main>", "    {/if}\n</main>")
        if relative == "src/routes/menu/connecting/Connecting.svelte":
            text = replace_once(text, "    let gameWindow: GameWindow | null = $state(null);",
                "    let gameWindow = $state<GameWindow | null>(null);")
        if relative == "src/routes/clickgui/Module.svelte":
            text = replace_once(text, '''        let moduleDescription = description;
        if (aliases.length > 0) {
            moduleDescription += ` (aka ${aliases.map(name => $spaceSeperatedNames ? convertToSpacedString(name) : name).join(", ")})`;
        }
''', "")
            if text.count("description: moduleDescription") != 2:
                raise ValueError("Module tooltip source changed")
            text = text.replace("description: moduleDescription", "description, moduleName: name, aliases")
        if relative == "src/routes/clickgui/Search.svelte":
            text = replace_once(text, '''        lowerName: string;
        lowerAliases: string[];''', "        names: string[];")
            text = replace_once(text, '''        lowerName: m.name.toLowerCase(),
        lowerAliases: m.aliases.map(a => a.toLowerCase()),''', "        names: $searchableModuleNames(m.name, m.aliases),")
            text = replace_once(text, '    // Cache lowercase names/aliases to avoid repeated toLowerCase() per keystroke.',
                '    // Rebuild display search names when the actual game language changes.')
            text = replace_once(text, '''        const pureQuery = query.toLowerCase().replaceAll(" ", "");

        filteredModules = searchableModules.filter(({ raw, lowerName, lowerAliases }) => {
            return lowerName.includes(pureQuery)
                || lowerAliases.some(a => a.includes(pureQuery));
        }).map(it => it.raw);''', '''        const pureQuery = query.toLocaleLowerCase().replace(/\\s+/g, "");

        filteredModules = searchableModules.filter(({names}) =>
            names.some(name => name.includes(pureQuery))
        ).map(it => it.raw);''')
            text = replace_once(text, "    function reset() {", '''    $: {
        searchableModules;
        if (query) filterModules(false);
    }

    function reset() {''')
            text = text.replace("(aka {aliases.map", '({$translate("aka")} {aliases.map')
            imports.update(["searchableModuleNames", "translate"])

        # All matches are labels/choice captions. Their bound values and paths
        # remain raw. Module names have their own dictionary to avoid collisions.
        expression = re.compile(r"\$?spaceSeperatedNames \? convertToSpacedString\(([A-Za-z_$][\w.$]*)\) : \1")
        def display_expression(match: re.Match) -> str:
            name = match[1]
            is_module = relative in ("src/routes/clickgui/Module.svelte", "src/routes/clickgui/Search.svelte",
                "src/routes/hud/elements/ArrayList.svelte", "src/routes/hud/elements/KeyBinds.svelte",
                "src/routes/hud/elements/tabgui/Module.svelte")
            if relative == "src/routes/clickgui/setting/MultiChooseSetting.svelte" and name == "choice":
                imports.add("translateChoice")
                return f"$translateChoice(cSetting, {name}, $spaceSeperatedNames)"
            helper = "translateModule" if is_module else "translate"
            imports.add(helper)
            return f"${helper}({name}, $spaceSeperatedNames)"
        text = expression.sub(display_expression, text)
        if "convertToSpacedString(" not in text:
            text = text.replace("convertToSpacedString, spaceSeperatedNames", "spaceSeperatedNames")

        if relative == "src/routes/clickgui/Panel.svelte":
            text = replace_once(text, '<span class="category">{category}</span>', '<span class="category">{$translate(category)}</span>')
            imports.add("translate")
        if relative == "src/routes/clickgui/tabs/Tabs.svelte":
            text = replace_once(text, "                {tab.title}", "                {$translate(tab.title)}")
            imports.add("translate")
        if relative == "src/routes/hud/elements/tabgui/Category.svelte":
            text = replace_once(text, "        {name}\n", "        {$translate(name)}\n")
            imports.add("translate")
        if relative == "src/routes/hud/elements/ArrayList.svelte":
            text = replace_once(text, '''    spaceSeperatedNames.subscribe(async () => {
        await updateEnabledModules();
    });''', '''    $: {
        $translateModule;
        $spaceSeperatedNames;
        updateEnabledModules();
    }''')
        if relative.startswith("src/routes/menu/common/") or relative == "src/routes/menu/title/buttons/ChildButton.svelte":
            if "{subTab.title}" in text:
                text = text.replace("{subTab.title}", "{$translate(subTab.title)}")
                imports.add("translate")
            if "{title}" in text and any(marker in relative for marker in ("/buttons/", "/modal/", "/menulist/", "/setting/", "/title/buttons/")):
                sections = re.split(r"(<(?:script|style)[^>]*>.*?</(?:script|style)>)", text, flags=re.S)
                for index in range(0, len(sections), 2):
                    # Presentation-only captions, including a shorthand native
                    # input placeholder. Icon/title identifiers are untouched.
                    sections[index] = sections[index].replace('placeholder={title}', 'placeholder={$translate(title)}')
                    sections[index] = re.sub(r">(\s*)\{title\}(\s*)<", r">\1{$translate(title)}\2<", sections[index])
                modified = "".join(sections)
                if modified != text:
                    imports.add("translate")
                    text = modified
        if relative == "src/routes/clickgui/Description.svelte":
            text = replace_once(text, '<div class="text">{data.description}</div>', '''<div class="text">{$translateDescription(data.description, data.moduleName)}{#if data.aliases?.length}
                    ({$translate("aka")} {data.aliases.map(name => $spaceSeperatedNames ? convertToSpacedString(name) : name).join(", ")})
                {/if}</div>''')
            text = text.replace('    import {fly}', '    import {convertToSpacedString, spaceSeperatedNames} from "../../theme/theme_config";\n    import {fly}')
            imports.update(["translate", "translateDescription"])
        if relative == "src/routes/clickgui/setting/common/GenericSetting.svelte":
            text = replace_once(text, '<div in:slide|global={{duration: 150, axis: "y"}}', '<div title={$translateSettingDescription(setting)} in:slide|global={{duration: 150, axis: "y"}}')
            text = replace_once(text, "Unsupported setting {setting.valueType}", '{$translate("Unsupported setting")} {setting.valueType}')
            imports.update(["translate", "translateSettingDescription"])
        if relative.endswith("/DrawerHudComponent.svelte"):
            text = replace_once(text, "{component.description}", "{$translateDescription(component.description)}")
            imports.add("translateDescription")
        if relative.endswith("/SwitchBindAction.svelte"):
            text = replace_once(text, ">{chosen}</span>", ">{$translate(chosen)}</span>")
            imports.add("translate")
        if relative.endswith("/BindDisplay.svelte"):
            text = replace_once(text, "    $: {\n", "    $: {\n        $localization.generation;\n")
            imports.add("localization")

        # Translate only literal presentation text and presentation attributes.
        # Never rewrite name/value/id props or strings inside component logic.
        sections = re.split(r"(<(?:script|style)[^>]*>.*?</(?:script|style)>)", text, flags=re.S)
        for index in range(0, len(sections), 2):
            def static_text(match: re.Match) -> str:
                value = match[1]
                key = value.strip()
                if key not in labels:
                    return match[0]
                imports.add("translate")
                leading = value[:len(value)-len(value.lstrip())]
                trailing = value[len(value.rstrip()):]
                return ">" + leading + "{$translate(" + json.dumps(key, ensure_ascii=False) + ")}" + trailing + "<"
            sections[index] = re.sub(r">([^<>{}\n]*[A-Za-z][^<>{}\n]*)<", static_text, sections[index])
            def static_attribute(match: re.Match) -> str:
                key = match[2]
                if key not in labels:
                    return match[0]
                imports.add("translate")
                return match[1] + "={$translate(" + json.dumps(key, ensure_ascii=False) + ")}"
            sections[index] = re.sub(r'(placeholder|aria-label|title|label|text)="([^"]+)"', static_attribute, sections[index])
        text = "".join(sections)
        text = add_import(text, relative, imports)
    if relative == "src/routes/clickgui/clickgui_store.ts":
        text = replace_once(text, "    description: string;", "    description: string;\n    moduleName?: string;\n    aliases?: string[];")
    if relative == "src/integration/events.ts":
        text = replace_once(text, "    clientLanguageChanged: void;", "    clientLanguageChanged: void;\n    languageChange: {gameLanguage: string; locale: string; generation: number};")
    # Keep Inter's Latin metrics; ask the system for real CJK glyphs.
    if relative.endswith((".svelte", ".scss")):
        text = text.replace('font-family: "Inter", sans-serif;', 'font-family: "Inter", "Microsoft YaHei", "Microsoft JhengHei", "PingFang SC", "PingFang TC", "Noto Sans CJK SC", "Noto Sans CJK TC", sans-serif;')
        text = text.replace('font-family: monospace;', 'font-family: "Cascadia Mono", "Microsoft YaHei", "Microsoft JhengHei", "Noto Sans Mono CJK SC", monospace;')
    return text


def main() -> None:
    labels = json.loads(DISPLAY.read_text(encoding="utf-8"))["labels"]
    output = []
    changed = []
    for path in sorted((SOURCE / "src").rglob("*")):
        if path.suffix not in (".svelte", ".ts", ".scss"):
            continue
        relative = path.relative_to(SOURCE).as_posix()
        original = path.read_bytes().decode("utf-8")
        modified = patch_source(relative, original.replace("\r\n", "\n"), labels)
        if modified == original:
            continue
        changed.append(relative)
        output.append(f"diff --git a/{relative} b/{relative}\n")
        for line in difflib.unified_diff(original.splitlines(keepends=True), modified.splitlines(keepends=True), fromfile="a/" + relative, tofile="b/" + relative):
            output.append(line)
            if not line.endswith("\n"):
                output.append("\n\\ No newline at end of file\n")
    PATCH.write_text("".join(output), encoding="utf-8", newline="\n")
    print(json.dumps({"patch": str(PATCH), "changedFiles": changed}, indent=2))


if __name__ == "__main__":
    main()
