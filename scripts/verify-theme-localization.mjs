// SPDX-License-Identifier: GPL-3.0-or-later
import assert from 'node:assert/strict';
import {readFile, readdir} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {
  description, englishSnapshot, label, moduleLabel, moduleSearchNames, searchTerm, settingDescription
} from '../native/liquidbounce-theme/src/integration/localization-core.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const folder = path.join(root, 'native/liquidbounce-interop/resources/localization');
const read = async name => JSON.parse(await readFile(path.join(folder, name + '.json'), 'utf8'));
const en = await read('en_us');
const cn = await read('zh_cn');
const tw = await read('zh_tw');
const cnDisplay = await read('display_zh_cn');
const twDisplay = await read('display_zh_tw');
const moduleKeys = await read('module_keys');
const english = {...englishSnapshot, moduleKeys, translations: {...cnDisplay.englishDescriptions, ...en}};
const chinese = {...english, gameLanguage: 'zh_cn', locale: 'zh_cn',
  ...cnDisplay, translations: {...english.translations, ...cnDisplay.descriptions, ...cn}};
const traditional = {...chinese, gameLanguage: 'zh_tw', locale: 'zh_tw',
  moduleNames: {...cnDisplay.moduleNames, ...twDisplay.moduleNames},
  labels: {...cnDisplay.labels, ...twDisplay.labels}, translations: {...chinese.translations, ...twDisplay.descriptions, ...tw}};

assert.equal(Object.keys(chinese.moduleNames).length, 236);
assert.deepEqual(Object.keys(chinese.moduleNames).sort(), Object.keys(moduleKeys).sort());
const module = {name: 'NoFOV', aliases: ['FOV'], category: 'Render', enabled: true};
const settings = {name: 'NoFOV', valueType: 'TOGGLEABLE', value: [
  {name: 'Mode', valueType: 'CHOICE', active: 'Custom', choices: {
    Constant: {name: 'Constant', value: []}, Custom: {name: 'Custom', value: [
      {name: 'Multiplier', value: 1.3, valueType: 'FLOAT'}
    ]}
  }}
]};
const original = JSON.stringify({module, settings});
assert.equal(moduleLabel(chinese, module.name, false), '固定视场角');
assert.equal(moduleLabel(chinese, module.name, true), '固定视场角');
assert.equal(label(chinese, module.category), '渲染');
assert.equal(label(chinese, 'Settings'), '设置');
assert.equal(label(traditional, 'Settings'), '設定');
assert.equal(label(english, 'Settings'), 'Settings');
assert.equal(label(chinese, settings.value[0].name), '模式');
assert.equal(label(chinese, settings.value[0].active), '自定义');
assert.equal(label(chinese, 'AnUntranslatedSetting', true), 'An Untranslated Setting');
assert.equal(label(chinese, 'AnUntranslatedSetting'), 'AnUntranslatedSetting');
assert.equal(moduleLabel(english, 'NoFOV', false), 'NoFOV');
assert.equal(moduleLabel(english, 'NoFOV', true), 'No FOV');
assert.equal(label(english, 'Search'), 'Search');
assert.equal(label(traditional, 'Search'), '搜尋');
for (const choice of ['PlacingWhileBreaking', 'AttackingWhileUsing', 'BreakingWhileUsing']) {
  assert.notEqual(label(chinese, choice), choice);
  assert.notEqual(label(traditional, choice), choice);
}
assert.equal(moduleLabel(traditional, 'NoBob'), '取消行走搖晃');
assert.equal(description(chinese, 'Old description', 'NoBob'), cn['liquidbounce.module.noBob.description']);
assert.equal(description(english, '曾经的中文描述', 'NoBob'), en['liquidbounce.module.noBob.description']);
assert.equal(description(chinese, 'liquidbounce.module.noBob.description'), cn['liquidbounce.module.noBob.description']);
const cachedSetting = {name: 'Hidden', key: 'liquidbounce.module.noBob.hidden',
  description: chinese.translations['liquidbounce.common.hidden.description']};
assert.equal(settingDescription(english, cachedSetting), english.translations['liquidbounce.common.hidden.description']);
assert.equal(settingDescription(chinese, cachedSetting), chinese.translations['liquidbounce.common.hidden.description']);
assert.notEqual(settingDescription(english, cachedSetting), cachedSetting.description);
for (const key of [
  "twerk.delay",
  "multiActions.actions",
  "noRotateSet.mode.resetRotation.rotations",
  "noRotateSet.mode.resetRotation.rotations.movementCorrection",
  "noRotateSet.mode.resetRotation.rotations.resetThreshold",
  "noRotateSet.mode.resetRotation.rotations.ticksUntilReset",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.linear.horizontalTurnSpeed",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.linear.verticalTurnSpeed",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.sigmoid.horizontalTurnSpeed",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.sigmoid.verticalTurnSpeed",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.sigmoid.steepness",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.sigmoid.midpoint",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.yawAcceleration",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.pitchAcceleration",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.dynamicAccel.enabled",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.dynamicAccel.coefDistance",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.dynamicAccel.yawCrosshairAccel",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.dynamicAccel.pitchCrosshairAccel",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.accelerationError.enabled",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.accelerationError.yawAccelError",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.accelerationError.pitchAccelError",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.constantError.enabled",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.constantError.yawConstantError",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.constantError.pitchConstantError",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.sigmoidDeceleration.enabled",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.sigmoidDeceleration.steepness",
  "noRotateSet.mode.resetRotation.rotations.angleSmooth.acceleration.sigmoidDeceleration.midpoint",
  'sprint.mode', 'sprint.ignore', 'sprint.stopOn', 'noSlowBreak.when',
  'antiBlind.doRender', 'antiBlind.fireOpacity', 'antiBlind.noParticles', 'antiBlind.skipTerrainLoading',
  'aspect.ratio', 'smoothCamera.enableFirstPOV', 'smoothCamera.resetOnPerspectiveChange',
  'smoothCamera.horizontalFactor', 'smoothCamera.verticalFactor', 'quickPerspectiveSwap.rearView',
  'crystalView.size', 'crystalView.yTranslate', 'crystalView.spinSpeed', 'crystalView.bounce',
  'autoF5.checkScreenHandlerType', 'autoF5.checkScreenHandlerType.types', 'autoF5.checkScreenHandlerType.filter',
  'autoF5.checkScreenTitle', 'autoF5.checkScreenTitle.titles', 'autoF5.checkScreenTitle.custom', 'autoF5.checkScreenTitle.filter',
  'fastPlace.cooldown', 'fastPlace.applyTo', 'fastPlace.startDelay', 'autoRespawn.delay',
  'keepSprint.motion', 'keepSprint.motionWhenHurt', 'keepSprint.hurtTime', 'keepSprint.chance',
  'airJump.mode', 'noPush.pushBy', 'autoF5.checkScreenHandlerType.enabled', 'autoF5.checkScreenTitle.enabled',
  'reach.entity.rangeIncrease', 'reach.entity.throughWallsRange', 'reach.blockRangeIncrease', 'autoFish.reelDelay', 'autoFish.sounds', 'autoFish.soundDistance.enabled', 'autoFish.soundDistance.maxDistance', 'autoFish.recastRod.enabled', 'autoFish.recastRod.delay', 'autoFish.autoCastRod.enabled', 'autoFish.autoCastRod.delay', 'noPose.noSwim', 'noPose.sneakHeight', 'cameraClip.cameraDistance', 'cameraClip.animation', 'cameraClip.animation.enabled', 'cameraClip.animation.speed', 'cameraClip.animation.easing', 'cameraClip.scrollAdjust', 'cameraClip.scrollAdjust.enabled', 'cameraClip.scrollAdjust.modifier', 'cameraClip.scrollAdjust.sensitivity', 'cameraClip.scrollAdjust.rememberScrolled', 'cameraClip.scrollAdjust.requireFreeLook', 'dankBobbing.motion'
]) {
  const setting = {name: key.split('.').at(-1), key: `liquidbounce.module.${key}`};
  const descriptionKey = setting.key + '.description';
  assert.equal(settingDescription(english, setting), cnDisplay.englishDescriptions[descriptionKey]);
  assert.equal(settingDescription(chinese, setting), cnDisplay.descriptions[descriptionKey]);
  assert.equal(settingDescription(traditional, setting), twDisplay.descriptions[descriptionKey]);
  assert.ok(settingDescription(chinese, setting));
  assert.notEqual(settingDescription(chinese, setting), settingDescription(english, setting));
}
// Owned dynamic AutoTool paths and full global/combat setting help.
for (const key of ["liquidbounce.module.hitbox.size", "liquidbounce.module.hitbox.applyToDebugHitbox", "liquidbounce.module.hitbox.applyToComponent", "liquidbounce.settings.targets.combat", "liquidbounce.settings.targets.visual", "liquidbounce.module.autoWeapon.preferred", "liquidbounce.module.autoWeapon.priority", "liquidbounce.module.autoWeapon.autoShieldBreak", "liquidbounce.module.autoWeapon.autoMace", "liquidbounce.module.autoWeapon.switchBack", "liquidbounce.module.autoWeapon.changeOn", "liquidbounce.module.autoTool", "liquidbounce.module.autoTool.toolSelector.dynamic.ignoreDurability", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.enabled", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.startDelay", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.clickDelay", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.closeDelay", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.missChance", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.slotMultiplier", "liquidbounce.module.autoTool.toolSelector.dynamic.considerInventory.constraints.requires", "liquidbounce.module.autoTool.toolSelector.static.slot", "liquidbounce.module.autoTool.filter", "liquidbounce.module.autoTool.blocks", "liquidbounce.module.autoTool.silkTouchHandler.enabled", "liquidbounce.module.autoTool.silkTouchHandler.filter", "liquidbounce.module.autoTool.silkTouchHandler.blocks", "liquidbounce.module.autoTool.swapBackDelay", "liquidbounce.module.autoTool.switchDelay", "liquidbounce.module.autoTool.requireSneaking", "liquidbounce.module.autoTool.notDuringCombat", "liquidbounce.module.autoTool.requireNearBed.enabled", "liquidbounce.module.autoTool.requireNearBed.distance", "liquidbounce.module.silentHotbar.noCooldownProgress"]) {
  const setting = {name:key.split('.').at(-1),key};
  const descriptionKey=key+'.description';
  assert.equal(settingDescription(english,setting),en[descriptionKey] ?? cnDisplay.englishDescriptions[descriptionKey]);
  assert.equal(settingDescription(chinese,setting),cn[descriptionKey] ?? cnDisplay.descriptions[descriptionKey]);
  assert.equal(settingDescription(traditional,setting),tw[descriptionKey] ?? twDisplay.descriptions[descriptionKey] ?? cn[descriptionKey] ?? cnDisplay.descriptions[descriptionKey]);
  assert.ok(settingDescription(chinese,setting));
}
const configFolder = path.join(root, 'native/liquidbounce-horizon/engine/src');
const configFiles = ['config.rs', ...(await readdir(path.join(configFolder, 'config'))).filter(name => name.endsWith('.rs')).map(name => 'config/' + name)];
const actualConfig = (await Promise.all(configFiles.map(name => readFile(path.join(configFolder, name), 'utf8')))).join('\n');
for (const [, key] of actualConfig.matchAll(/"(liquidbounce\.(?:module|common)\.[^"]+)"/g)) {
  // Rust format! templates describe a family of keys, not a literal resource.
  if (key.includes('{') || key.includes('}')) continue;
  const descriptionKey = key.endsWith('.description') ? key : key + '.description';
  for (const snapshot of [english, chinese, traditional]) {
    assert.ok(snapshot.translations[descriptionKey], `Missing ${snapshot.locale} setting help: ${descriptionKey}`);
  }
}
// These complete families are formed dynamically by the actual config adapters.
for (const prefix of ['liquidbounce.module.antiAFK.mode.', 'liquidbounce.module.smartEat.']) {
  const keys = Object.keys(cnDisplay.englishDescriptions).filter(key => key.startsWith(prefix));
  assert.equal(keys.length, prefix.includes('antiAFK') ? 36 : 14);
  for (const key of keys) {
    for (const snapshot of [english, chinese, traditional]) {
      assert.ok(snapshot.translations[key], `Missing ${snapshot.locale} dynamic setting help: ${key}`);
    }
    assert.notEqual(chinese.translations[key], english.translations[key]);
    assert.notEqual(traditional.translations[key], english.translations[key]);
  }
}
for (const query of ['固定视场角', 'No FOV', 'fov']) {
  assert.ok(moduleSearchNames(chinese, module.name, module.aliases).some(name => name.includes(searchTerm(query))));
}
assert.equal(JSON.stringify({module, settings}), original);
console.log(JSON.stringify({passed: true, translatedModuleNames: Object.keys(chinese.moduleNames).length,
  simplifiedLabels: Object.keys(cnDisplay.labels).length,
  traditionalLabels: Object.keys(twDisplay.labels).length,
  ownedSettingDescriptions: Object.keys(cnDisplay.descriptions).length,
  checks: ['Chinese display', 'traditional display', 'Settings heading', 'new module setting help', 'English restoration', 'spaced English fallback',
    'Chinese and English search', 'upstream descriptions', 'stable module/setting/choice identities']}));
