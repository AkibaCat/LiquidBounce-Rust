#!/usr/bin/env python3
"""Transactional scoped choice captions: armor Chest must not rename container Chest."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
    p=Plan();core='../../../liquidbounce-theme/src/integration/localization-core.ts';ui='../../../liquidbounce-theme/src/integration/localization.ts';generator='../../../../scripts/make-theme-localization-patch.py'
    a='export function moduleLabel(snapshot: LocalizationSnapshot, name: string, spaced = false): string {'
    p.edit(core,a,'''// A setting's raw choice stays unchanged; its key disambiguates shared words.
export function choiceLabel(snapshot: LocalizationSnapshot, setting: {key?: string}, name: string, spaced = false): string {
    const key = setting.key && `${setting.key}.${name}`;
    return (key && snapshot.labels[key]) || label(snapshot, name, spaced);
}

'''+a)
    p.edit(ui,'    description, englishSnapshot, label, moduleLabel, moduleSearchNames, settingDescription,','    choiceLabel, description, englishSnapshot, label, moduleLabel, moduleSearchNames, settingDescription,')
    a='export const translateModule = derived(localization, snapshot =>'
    p.edit(ui,a,'''export const translateChoice = derived(localization, snapshot =>
    (setting: {key?: string}, name: string, spaced = false) => choiceLabel(snapshot, setting, name, spaced));
'''+a)
    a='''            helper = "translateModule" if is_module else "translate"
            imports.add(helper)
            return f"${helper}({name}, $spaceSeperatedNames)"'''
    p.edit(generator,a,'''            if relative == "src/routes/clickgui/setting/MultiChooseSetting.svelte" and name == "choice":
                imports.add("translateChoice")
                return f"$translateChoice(cSetting, {name}, $spaceSeperatedNames)"
            helper = "translateModule" if is_module else "translate"
            imports.add(helper)
            return f"${helper}({name}, $spaceSeperatedNames)"''')
    return p
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args();p=prepare()
    if not args.check:p.commit()
    print(f'BedPlates scoped choice labels {"validated"if args.check else"integrated"}: {len(p.changed)} shared files')
