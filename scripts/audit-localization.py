#!/usr/bin/env python3
"""Audit literal configuration captions without executing the Kotlin client.

This is deliberately a static inventory, not a claim of native module parity or
an exact evaluated runtime configuration tree. Dynamic names/callbacks and
ambiguous helper references remain visible in the generated report.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re

PROJECT = Path(__file__).resolve().parent.parent
SOURCE = PROJECT / "upstream/LiquidBounce"
KOTLIN = SOURCE / "src/main/kotlin"
MODULES = KOTLIN / "net/ccbluex/liquidbounce/features/module/modules"
RESOURCES = PROJECT / "native/liquidbounce-interop/resources/localization"
OUTPUT = PROJECT / "research/localization-coverage.json"
REPORT = PROJECT / "research/localization-coverage.md"
BUILDERS = set("boolean int intRange float floatRange double doubleRange text textList regex regexList color colorList vector vec2f vec3i vec3d curve easing enumChoice enumChoiceList multiEnumChoice enumList choices choose chooseList multiChoose key keybind bind item items block blocks file blockList itemList registryList registryMutableList sounds mobEffects enchantments c2sPackets s2cPackets entityTypes percentageChance modes".split())
BASE_GROUPS = {"ValueGroup", "ToggleableValueGroup", "Mode", "ModeValueGroup", "ClientModule"}
TOKEN = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"""[\s\S]*?"""|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[A-Za-z_$][\w$]*|\S')


def tokenize(text: str) -> list[tuple[str, int]]:
    return [(match[0], match.start()) for match in TOKEN.finditer(text)
            if not match[0].startswith(("//", "/*"))]


def literal(token: str) -> str | None:
    if not token.startswith('"') or "$" in token:
        return None
    if token.startswith('"""'):
        return token[3:-3]
    try:
        return json.loads(token)
    except ValueError:
        return None


def close(tokens: list[tuple[str, int]], start: int) -> int:
    pairs = {"(": ")", "{": "}", "[": "]", "<": ">"}
    stack = [pairs[tokens[start][0]]]
    for index in range(start + 1, len(tokens)):
        value = tokens[index][0]
        if value in "({[":
            stack.append(pairs[value])
        elif value == stack[-1]:
            stack.pop()
            if not stack:
                return index
    return start


def arguments(tokens: list[tuple[str, int]], start: int, end: int) -> list[list[tuple[str, int]]]:
    result: list[list[tuple[str, int]]] = []
    current: list[tuple[str, int]] = []
    depth = 0
    for token in tokens[start + 1:end]:
        value = token[0]
        if value == "," and depth == 0:
            result.append(current)
            current = []
            continue
        current.append(token)
        if value in "({[<":
            depth += 1
        elif value in ")}]>":
            depth -= 1
    if current:
        result.append(current)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Fail if saved reports differ from the current inventory")
    parser.add_argument("--fail-on-missing", action="store_true", help="Also fail on any uncovered simplified Chinese literal")
    args = parser.parse_args()
    catalog = json.loads((PROJECT / "research/module-parity.json").read_text(encoding="utf-8-sig"))
    displays = {locale: json.loads((RESOURCES / f"display_{locale}.json").read_text(encoding="utf-8")) for locale in ("zh_cn", "zh_tw")}
    preserve_file = RESOURCES / "display_preserve.json"
    preserve = json.loads(preserve_file.read_text(encoding="utf-8")) if preserve_file.exists() else {}
    files = {}
    declared = defaultdict(list)
    parents = defaultdict(set)
    for path in sorted(KOTLIN.rglob("*.kt")):
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(SOURCE).as_posix()
        tokens = tokenize(text)
        package = re.search(r"(?m)^package\s+([\w.]+)", text)
        package = package[1] if package else ""
        symbols = set()
        for index, (value, _) in enumerate(tokens[:-1]):
            if value not in ("class", "object", "interface") or not re.match(r"^[A-Za-z_]", tokens[index + 1][0]):
                continue
            name = tokens[index + 1][0]
            symbols.add(name)
            declared[name].append(relative)
            cursor = index + 2
            # Constructor parameters can contain lambdas; jump complete brackets.
            if cursor < len(tokens) and tokens[cursor][0] == "<":
                cursor = close(tokens, cursor) + 1
            if cursor < len(tokens) and tokens[cursor][0] == "(":
                cursor = close(tokens, cursor) + 1
            if cursor < len(tokens) and tokens[cursor][0] == ":":
                for parent, _ in tokens[cursor + 1:cursor + 5]:
                    if re.match(r"^[A-Za-z_]", parent):
                        parents[name].add(parent)
                        break
        imports = {}
        for match in re.finditer(r"(?m)^import\s+([\w.]+)(?:\s+as\s+(\w+))?", text):
            imports[match[2] or match[1].split(".")[-1]] = match[1]
        files[relative] = {"text": text, "tokens": tokens, "package": package, "imports": imports,
                           "symbols": symbols, "moduleDirectory": path.is_relative_to(MODULES)}
    group_types = set(BASE_GROUPS)
    while True:
        additions = {name for name, bases in parents.items() if bases & group_types} - group_types
        if not additions:
            break
        group_types |= additions
    callable_names = BUILDERS | group_types | {"tree", "treeAll"}
    records = []
    tagged_types = set()
    dynamic = []
    file_refs = defaultdict(set)
    file_records = defaultdict(list)
    file_dynamic = defaultdict(list)
    for relative, data in files.items():
        tokens = data["tokens"]
        text = data["text"]
        for index, (name, position) in enumerate(tokens[:-1]):
            if name not in callable_names:
                continue
            # Exclude function declarations. Class super-constructor calls stay.
            if index and tokens[index - 1][0] == "fun":
                continue
            cursor = index + 1
            if tokens[cursor][0] == "<":
                cursor = close(tokens, cursor) + 1
            if cursor >= len(tokens) or tokens[cursor][0] != "(":
                continue
            end = close(tokens, cursor)
            if end == cursor:
                continue
            call_args = arguments(tokens, cursor, end)
            segment_end = end
            if end + 1 < len(tokens) and tokens[end + 1][0] == "{":
                segment_end = close(tokens, end + 1)
            file_refs[relative] |= {value for value, _ in tokens[index:segment_end + 1] if re.match(r"^[A-Za-z_]\w*$", value)}
            line = text.count("\n", 0, position) + 1
            if name in {"tree", "treeAll"}:
                # A callback/name cannot be evaluated by this static scanner.
                if not any(value in declared for arg in call_args for value, _ in arg):
                    issue = {"source": relative, "line": line, "call": name, "expression": text[tokens[cursor][1]:tokens[end][1] + 1], "reason": "tree reference requires runtime or Kotlin resolution"}
                    file_dynamic[relative].append(len(dynamic))
                    dynamic.append(issue)
                continue
            if name in {"modes", "choices"} and (
                any(value == ":" and i + 1 < len(tokens) and tokens[i + 1][0] == ":"
                    for i, (value, _) in enumerate(tokens[index:segment_end + 1], start=index))
                or (segment_end != end)
                or any(value == "{" for value, _ in tokens[cursor:end])
            ):
                issue = {"source": relative, "line": line, "call": name,
                         "expression": text[tokens[cursor][1]:tokens[segment_end][1] + 1][:600],
                         "reason": "mode factory/callback requires evaluated runtime enumeration"}
                file_dynamic[relative].append(len(dynamic))
                dynamic.append(issue)
            label_value = None
            # Event-listener first overloads put name in their second argument.
            candidate_args = call_args[:2] if name in {"modes", "ToggleableValueGroup", "ModeValueGroup"} else call_args[:1]
            for arg in candidate_args:
                values = [value for value, _ in arg]
                if len(values) >= 3 and values[:2] == ["name", "="]:
                    values = values[2:]
                if len(values) == 1:
                    label_value = literal(values[0])
                    if label_value is not None:
                        break
            if label_value is None:
                # Framework declarations forward `name`; omit those internals,
                # but retain real dynamic expressions used by module/helpers.
                if data["moduleDirectory"] or not relative.startswith("src/main/kotlin/net/ccbluex/liquidbounce/config/"):
                    issue = {"source": relative, "line": line, "call": name,
                             "expression": text[tokens[cursor][1]:tokens[end][1] + 1][:600],
                             "reason": "nonliteral or interpolated configuration name"}
                    file_dynamic[relative].append(len(dynamic))
                    dynamic.append(issue)
                continue
            kind = "module" if name == "ClientModule" else "groupOrMode" if name in group_types else "setting"
            record = {"label": label_value, "kind": kind, "call": name, "source": relative, "line": line}
            file_records[relative].append(len(records))
            records.append(record)
        # Tagged enums expose their literal tag values as choice captions. They
        # can live outside module directories, so scan shared Kotlin sources too.
        for match in re.finditer(r"\benum\s+class\s+(\w+)", text):
            start_index = next((i for i, (_, pos) in enumerate(tokens) if pos >= match.start()), None)
            if start_index is None:
                continue
            body = next((i for i in range(start_index, len(tokens)) if tokens[i][0] == "{"), None)
            if body is None:
                continue
            header = text[match.start():tokens[body][1]]
            if "Tagged" not in header and not re.search(r"override\s+val\s+tag\b", header):
                continue
            tagged_types.add(match[1])
            parameter_start = next((i for i in range(start_index, body) if tokens[i][0] == "("), None)
            tag_argument = 0
            if parameter_start is not None:
                parameter_end = close(tokens, parameter_start)
                parameters = arguments(tokens, parameter_start, parameter_end)
                tag_argument = next((i for i, parameter in enumerate(parameters)
                                     if any(value == "tag" for value, _ in parameter)), 0)
            enum_end = close(tokens, body)
            enum_tokens = tokens[body + 1:enum_end]
            depth = 0
            for offset, (value, pos) in enumerate(enum_tokens[:-2]):
                if value == ";" and depth == 0:
                    break
                if value in "({[":
                    depth += 1
                elif value in ")}]":
                    depth -= 1
                if depth != 0 or not re.match(r"^[A-Za-z_][\w]*$", value) or enum_tokens[offset + 1][0] != "(":
                    continue
                entry_start = body + 1 + offset + 1
                entry_end = close(tokens, entry_start)
                entry_arguments = arguments(tokens, entry_start, entry_end)
                tag_tokens = entry_arguments[tag_argument] if len(entry_arguments) > tag_argument else []
                tag_values = [value for value, _ in tag_tokens]
                if tag_values[:2] == ["tag", "="]:
                    tag_values = tag_values[2:]
                caption = literal(tag_values[0]) if len(tag_values) == 1 else None
                if caption is not None:
                    record = {"label": caption, "kind": "enumChoice", "call": match[1], "source": relative, "line": text.count("\n", 0, pos) + 1}
                    file_records[relative].append(len(records))
                    records.append(record)
                else:
                    issue = {"source": relative, "line": text.count("\n", 0, pos) + 1,
                             "call": match[1] + "." + value,
                             "expression": text[tokens[entry_start][1]:tokens[entry_end][1] + 1][:600],
                             "reason": "nonliteral or computed Tagged enum caption"}
                    file_dynamic[relative].append(len(dynamic))
                    dynamic.append(issue)
    # Link only configuration-call arguments, not arbitrary gameplay references.
    graph = defaultdict(set)
    unresolved = defaultdict(list)
    child_types = (group_types | tagged_types) - BASE_GROUPS
    for relative, refs in file_refs.items():
        data = files[relative]
        for symbol in sorted(refs):
            # A setting's default value may reference arbitrary game objects.
            # Those are not configuration children. Framework base classes are
            # also not children: following them would import the whole client.
            if symbol not in child_types:
                continue
            candidates = declared.get(symbol, [])
            if not candidates:
                continue
            if relative in candidates:
                continue
            imported = data["imports"].get(symbol)
            if imported:
                matches = [candidate for candidate in candidates if imported.startswith(files[candidate]["package"] + ".")]
            else:
                matches = [candidate for candidate in candidates if files[candidate]["package"] == data["package"]]
            if not matches and len(candidates) == 1:
                matches = candidates
            if len(matches) == 1:
                # Module parents identify their groups but must not pull another
                # module's entire tree into an unrelated root.
                if not symbol.startswith("Module"):
                    graph[relative].add(matches[0])
            elif len(candidates) > 1 and symbol in group_types:
                unresolved[relative].append({"symbol": symbol, "candidates": sorted(candidates)})
    def caption_status(label: str, locale: str) -> str:
        if label in displays[locale]["labels"] or label in displays[locale]["moduleNames"]:
            return "translated"
        if label in preserve:
            return "intentionalEnglish"
        if locale == "zh_tw" and (label in displays["zh_cn"]["labels"] or label in displays["zh_cn"]["moduleNames"]):
            return "simplifiedFallback"
        return "missing"
    for record in records:
        record["coverage"] = {locale: caption_status(record["label"], locale) for locale in displays}
    module_reports = []
    reached = set()
    for module in catalog["modules"]:
        root = module["source"]
        visited = set()
        queue = [root]
        while queue:
            item = queue.pop()
            if item in visited:
                continue
            visited.add(item)
            queue.extend(graph[item] - visited)
        reached |= visited
        indices = sorted({index for item in visited for index in file_records[item]})
        dynamic_indices = sorted({index for item in visited for index in file_dynamic[item]})
        counts = {locale: dict(sorted(Counter(records[index]["coverage"][locale] for index in indices).items())) for locale in displays}
        module_reports.append({"symbol": module["symbol"], "source": root, "configurationSources": sorted(visited),
                               "literalRecordIndices": indices, "coverage": counts,
                               "dynamicRecordIndices": dynamic_indices,
                               "ambiguousReferences": [{"source": item, **value} for item in sorted(visited) for value in unresolved[item]],
                               "scope": "file-level static configuration-reference closure; not evaluated runtime tree"})
    relevant = sorted({index for file in reached if files[file]["moduleDirectory"] or file_records[file] for index in file_records[file]})
    module_directory_indices = [index for index, record in enumerate(records) if files[record["source"]]["moduleDirectory"]]
    # This upper-bound vocabulary catches modes/helpers which static callbacks
    # do not connect to a module root, including unregistered module sources.
    inventory_indices = sorted(set(relevant) | set(module_directory_indices))
    summary = {"registeredModulesAudited": len(module_reports), "allKotlinFilesScanned": len(files),
               "moduleDirectoryKotlinFiles": sum(data["moduleDirectory"] for data in files.values()),
               "configGroupTypesRecognized": len(group_types), "literalDeclarations": len(inventory_indices),
               "distinctLiteralCaptions": len({records[i]["label"] for i in inventory_indices}),
               "dynamicDeclarations": len({index for file in reached | {f for f in files if files[f]["moduleDirectory"]} for index in file_dynamic[file]}),
               "coverage": {locale: dict(sorted(Counter(records[i]["coverage"][locale] for i in inventory_indices).items())) for locale in displays}}
    missing = {locale: sorted({records[i]["label"] for i in inventory_indices if records[i]["coverage"][locale] == "missing"}) for locale in displays}
    native_source = (PROJECT / "native/liquidbounce-horizon/engine/src/config.rs").read_text(encoding="utf-8")
    native_module_names = sorted(set(re.findall(r'^\s*\w+\s*=>\s*\("([^"]+)"', native_source, re.M)))
    native_labels = sorted(set(re.findall(r'tree\.(?:leaf|ranged|group|choices|multi)\(\s*"([^"]+)"', native_source)))
    native_caption_coverage = {"moduleNames": native_module_names, "literalSettingAndChoiceCaptions": native_labels,
                               "missing": {locale: {"moduleNames": [name for name in native_module_names if caption_status(name, locale) == "missing"],
                                                     "literalSettingAndChoiceCaptions": [name for name in native_labels if caption_status(name, locale) == "missing"]}
                                           for locale in displays},
                               "scope": "literal native registry/serializer captions only; dynamic serializer paths are verified separately"}
    kotlin_digest = hashlib.sha256()
    for relative, data in sorted(files.items()):
        kotlin_digest.update(relative.encode("utf-8") + b"\0" + data["text"].encode("utf-8") + b"\0")
    payload = {"schemaVersion": 1, "upstreamCommit": catalog["upstreamCommit"],
               "method": "Kotlin lexical literal scan; constructor inheritance, setting DSL, Tagged enums, tree/mode argument references; comments excluded",
               "limitations": ["No Kotlin execution or compiler symbol/type resolution", "Configuration-reference closure is file-level and can include unrelated declarations in a helper file", "Dynamic constructor names, generated modes and callbacks require evaluated-runtime verification", "Module-directory vocabulary includes unregistered modules and unattached helper/mode files; this is an upper bound", "Coverage measures display vocabulary, never gameplay implementation or exact native config-tree parity"],
               "inputs": {"catalogSha256": hashlib.sha256((PROJECT / "research/module-parity.json").read_bytes()).hexdigest(),
                          "scannedKotlinNormalizedTextSha256": kotlin_digest.hexdigest(),
                          "nativeConfigSha256": hashlib.sha256(native_source.encode("utf-8")).hexdigest(),
                          "intentionalEnglishSha256": hashlib.sha256(preserve_file.read_bytes()).hexdigest() if preserve_file.exists() else None,
                          **{f"display_{locale}Sha256": hashlib.sha256((RESOURCES / f"display_{locale}.json").read_bytes()).hexdigest() for locale in displays}},
               "summary": summary, "missingDistinctCaptions": missing, "intentionalEnglish": preserve,
               "nativeCaptionCoverage": native_caption_coverage,
               "inventoryRecordIndices": inventory_indices, "modules": module_reports,
               "literalRecords": records, "dynamicRecords": dynamic}
    report = "# Static Chinese configuration-caption coverage\n\n"
    report += f"Pinned upstream: `{catalog['upstreamCommit']}`. Audited {len(module_reports)} registered module roots and {summary['moduleDirectoryKotlinFiles']} module-directory Kotlin files.\n\n"
    report += f"The upper-bound inventory contains {summary['literalDeclarations']} literal declarations / {summary['distinctLiteralCaptions']} distinct captions.\n\n"
    report += "| Locale | Translated declarations | Deliberately retained English | Simplified fallback | Missing declarations | Missing distinct captions |\n| --- | ---: | ---: | ---: | ---: | ---: |\n"
    for locale, counts in summary["coverage"].items():
        report += f"| {locale} | {counts.get('translated', 0)} | {counts.get('intentionalEnglish', 0)} | {counts.get('simplifiedFallback', 0)} | {counts.get('missing', 0)} | {len(missing[locale])} |\n"
    report += f"\n{summary['dynamicDeclarations']} nonliteral/tree declarations remain explicitly recorded for Kotlin/runtime resolution. The per-module report includes reachable configuration source files, literal indices, dynamic indices, and ambiguous references.\n\n"
    report += "This report is a lexical audit of display vocabulary. It does not prove an exact evaluated runtime configuration tree or completed native gameplay implementations. File-level helper closure can include sibling declarations; module-directory coverage also includes unattached helpers and three unregistered modules.\n\n"
    report += "Regenerate: `python scripts/audit-localization.py`. Check freshness: `python scripts/audit-localization.py --check`. Require no missing simplified literals: `python scripts/audit-localization.py --check --fail-on-missing`.\n"
    outputs = {OUTPUT: json.dumps(payload, ensure_ascii=False, indent=2) + "\n", REPORT: report}
    if args.check:
        changed = [path for path, content in outputs.items() if not path.exists() or path.read_text(encoding="utf-8") != content]
        if changed:
            raise SystemExit("Stale audit: " + ", ".join(str(path.relative_to(PROJECT)) for path in changed))
    else:
        for path, content in outputs.items():
            path.write_text(content, encoding="utf-8")
    print(json.dumps({"passed": not (args.fail_on_missing and missing["zh_cn"]), **summary,
                      "missingDistinctCaptions": {locale: len(values) for locale, values in missing.items()}}, ensure_ascii=False))
    if args.fail_on_missing and missing["zh_cn"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
