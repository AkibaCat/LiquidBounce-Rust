// SPDX-License-Identifier: GPL-3.0-or-later
// Root executes this only after the two transactions and locale updater.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {choiceLabel,englishSnapshot,label} from '../liquidbounce-theme/src/integration/localization-core.ts';
const setting={key:'liquidbounce.module.bedPlates.ignoreSelfBed.color.slots'};
for(const code of ['zh_cn','zh_tw']){
    const data=JSON.parse(await readFile(new URL(`../liquidbounce-interop/resources/localization/display_${code}.json`,import.meta.url),'utf8'));
    const snapshot={...englishSnapshot,...data};
    assert.equal(choiceLabel(snapshot,setting,'Chest'),'胸部');
    assert.equal(label(snapshot,'Chest'),code==='zh_cn'?'箱子':'儲物箱');
    assert.equal(choiceLabel(snapshot,{key:'another.module.setting'},'Chest'),data.labels.Chest);
    assert.equal(choiceLabel(snapshot,{},'Feet'),label(snapshot,'Feet'));
    assert.equal(choiceLabel(snapshot,setting,'UnknownChoice',true),'Unknown Choice');
}
assert.equal(choiceLabel(englishSnapshot,setting,'Chest'),'Chest');
assert.equal(setting.key,'liquidbounce.module.bedPlates.ignoreSelfBed.color.slots');
console.log('BedPlates scoped choices: Simplified/Traditional armor captions, generic container labels, English fallback and identifier preservation passed');
