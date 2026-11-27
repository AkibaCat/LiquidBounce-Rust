import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(project, 'upstream', 'LiquidBounce');
const tree = JSON.parse(await readFile(path.join(project, 'research', 'upstream-tree.json'), 'utf8'));
const managerPath = 'src/main/kotlin/net/ccbluex/liquidbounce/features/module/ModuleManager.kt';
const manager = await readFile(path.join(source, managerPath), 'utf8');
const builtin = manager.match(/val builtin = arrayOf\(([\s\S]*?)\n\s*\)/)?.[1];
if (!builtin) throw new Error('Could not locate the upstream builtin module registry.');

const moduleDirectory = 'src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/';
const kotlinFiles = tree.tree.filter(entry => entry.type === 'blob'
  && entry.path.startsWith(moduleDirectory) && entry.path.endsWith('.kt'));
const declarations = new Map();
// Pure source-derived behavior is tracked separately from functioning game modules.
const implementations = {
  ModuleZoom: 'native/liquidbounce-behaviors/src/zoom.rs',
  ModuleFullBright: 'native/liquidbounce-behaviors/src/full_bright.rs',
  ModuleNoFov: 'native/liquidbounce-behaviors/src/no_fov.rs',
  ModuleNoBob: 'native/liquidbounce-horizon/engine/src/runtime.rs',
  ModuleNoHurtCam: 'native/liquidbounce-horizon/engine/src/runtime.rs',
  ModuleNoJumpDelay: 'native/liquidbounce-behaviors/src/movement_query.rs',
  ModuleAntiBounce: 'native/liquidbounce-behaviors/src/movement_query.rs',
  ModuleAntiLevitation: 'native/liquidbounce-behaviors/src/movement_query.rs',
  ModuleLiquidPlace: 'native/liquidbounce-behaviors/src/interaction_query.rs',
  ModulePortalMenu: 'native/liquidbounce-behaviors/src/interaction_query.rs',
  ModuleAntiReducedDebugInfo: 'native/liquidbounce-behaviors/src/interaction_query.rs',
  ModuleNoMissCooldown: 'native/liquidbounce-behaviors/src/no_miss_cooldown.rs',
  ModuleAbortBreaking: 'native/liquidbounce-behaviors/src/interaction_query.rs',
  ModuleNoSwing: 'native/liquidbounce-behaviors/src/no_swing.rs',
  ModuleMoreCarry: 'native/liquidbounce-behaviors/src/more_carry.rs',
  ModuleNoPitchLimit: 'native/liquidbounce-behaviors/src/no_pitch_limit.rs',
  ModuleNoInterpolation: 'native/liquidbounce-behaviors/src/no_interpolation.rs',
  ModuleNoSlowBreak: 'native/liquidbounce-behaviors/src/no_slow_break.rs',
  ModuleNoSlotSet: 'native/liquidbounce-behaviors/src/no_slot_set.rs',
  ModuleSprint: 'native/liquidbounce-behaviors/src/sprint.rs',
  ModuleAntiBlind: 'native/liquidbounce-behaviors/src/visual_environment.rs',
  ModuleAirJump: 'native/liquidbounce-behaviors/src/air_jump.rs',
  ModuleNoPush: 'native/liquidbounce-behaviors/src/no_push.rs',
  ModuleAspect: 'native/liquidbounce-behaviors/src/camera_control.rs',
  ModuleSmoothCamera: 'native/liquidbounce-behaviors/src/camera_control.rs',
  ModuleQuickPerspectiveSwap: 'native/liquidbounce-behaviors/src/camera_control.rs',
  ModuleAutoRespawn: 'native/liquidbounce-behaviors/src/auto_respawn.rs',
  ModuleKeepSprint: 'native/liquidbounce-behaviors/src/keep_sprint.rs',
  ModuleCrystalView: 'native/liquidbounce-behaviors/src/crystal_view.rs',
  ModuleFastPlace: 'native/liquidbounce-behaviors/src/fast_place.rs',
  ModuleAutoF5: 'native/liquidbounce-behaviors/src/auto_f5.rs',
  ModuleAutoFish: 'native/liquidbounce-behaviors/src/auto_fish.rs',
  ModuleReach: 'native/liquidbounce-behaviors/src/reach.rs',
  ModuleNoPose: 'native/liquidbounce-behaviors/src/no_pose.rs',
  ModuleNoEntityInteract: 'native/liquidbounce-behaviors/src/no_entity_interact.rs',
  ModuleNoBlockInteract: 'native/liquidbounce-behaviors/src/no_block_interact.rs',
  ModuleNoRotateSet: 'native/liquidbounce-behaviors/src/no_rotate_set.rs',
  ModuleCameraClip: 'native/liquidbounce-behaviors/src/camera_clip.rs',
  ModuleDankBobbing: 'native/liquidbounce-behaviors/src/dank_bobbing.rs',
  ModuleMultiActions: 'native/liquidbounce-behaviors/src/multi_actions.rs',
  ModuleTwerk: 'native/liquidbounce-behaviors/src/twerk.rs',
  ModuleHitbox: 'native/liquidbounce-behaviors/src/hitbox.rs',
  ModuleAutoWeapon: 'native/liquidbounce-behaviors/src/auto_weapon.rs',
  ModuleChams: 'native/liquidbounce-behaviors/src/chams.rs',
  ModuleAutoTool: 'native/liquidbounce-behaviors/src/auto_tool.rs',
  ModuleAntiAFK: 'native/liquidbounce-behaviors/src/anti_afk.rs',
  ModuleTridentBoost: 'native/liquidbounce-behaviors/src/trident_boost.rs',
  ModuleDerp: 'native/liquidbounce-behaviors/src/derp.rs',
  ModuleSleepWalker: 'native/liquidbounce-horizon/sleep-walker-host.rs',
  ModuleFreeLook: 'native/liquidbounce-behaviors/src/free_look.rs',
  ModuleHandDerp: 'native/liquidbounce-behaviors/src/hand_derp.rs',
  ModuleVehicleBoost: 'native/liquidbounce-behaviors/src/vehicle_boost.rs',
  ModuleBlockBounce: 'native/liquidbounce-behaviors/src/block_bounce.rs',
  ModuleSmartEat: 'native/liquidbounce-behaviors/src/smart_eat.rs',
  ModuleSkinDerp: 'native/liquidbounce-behaviors/src/skin_derp.rs',
  ModuleEntityControl: 'native/liquidbounce-behaviors/src/entity_control.rs',
  ModuleSnapTap: 'native/liquidbounce-behaviors/src/snap_tap.rs',
  ModuleBlockWalk: 'native/liquidbounce-behaviors/src/block_walk.rs',
  ModuleGhostHand: 'native/liquidbounce-behaviors/src/ghost_hand.rs',
  ModuleExtendedFirework: 'native/liquidbounce-behaviors/src/extended_firework.rs',
  ModuleStrafe: 'native/liquidbounce-behaviors/src/strafe.rs',
  ModuleElytraRecast: 'native/liquidbounce-behaviors/src/elytra_recast.rs',
  ModuleFastBreak: 'native/liquidbounce-behaviors/src/fast_break.rs',
  ModuleVomit: 'native/liquidbounce-behaviors/src/vomit.rs',
  ModuleVehicleOneHit: 'native/liquidbounce-behaviors/src/vehicle_one_hit.rs',
  ModuleTimer: 'native/liquidbounce-behaviors/src/timer.rs',
  ModuleBetterInventory: 'native/liquidbounce-behaviors/src/better_inventory.rs',
  ModuleTerrainSpeed: 'native/liquidbounce-behaviors/src/terrain_speed.rs',
  ModuleAutoBreak: 'native/liquidbounce-behaviors/src/auto_break.rs',
  ModuleHighJump: 'native/liquidbounce-behaviors/src/high_jump.rs',
  ModuleItemScroller: 'native/liquidbounce-behaviors/src/item_scroller.rs',
  ModuleAntiHunger: 'native/liquidbounce-behaviors/src/anti_hunger.rs',
  ModuleSilentHotbar: 'native/liquidbounce-behaviors/src/silent_hotbar_render.rs',
};
const optionalJson = async relative => {
  try { return JSON.parse((await readFile(path.join(project, relative), 'utf8')).replace(/^\uFEFF/, '')); }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
};
// Evidence belongs to the exact compiled sources. A later native edit returns
// the affected build to pending until its actual integration run passes.
async function integrationEvidence() {
  const relative = 'research/native-integration-validation.json';
  const evidence = await optionalJson(relative);
  const build = await optionalJson('build/integrated-build.json');
  if (evidence && ('validationScope' in evidence || 'fullIntegrationPassed' in evidence)
      && (evidence.validationScope !== 'full_integration' || evidence.fullIntegrationPassed !== true)) return null;
  if (evidence && 'screenshotPendingVisualReview' in evidence
      && (evidence.uiVisualReview?.passed !== true
          || evidence.screenshotPendingVisualReview?.status !== 'reviewed_pass')) return null;
  if (!evidence?.passed || !evidence.verificationPassed || !build?.passed
      || !Object.values(evidence.checks ?? {}).every(check => check.passed)
      || !build.staging?.nativeSourceSha256
      || !Object.keys(build.staging.nativeSourceFiles ?? {}).length
      || evidence.runtimeProvenance?.integratedBuild?.executableSha256 !== build.executableSha256
      || evidence.runtimeProvenance?.executableOnDisk?.sha256 !== build.executableSha256
      || evidence.runtimeProvenance?.integratedBuild?.staging?.nativeSourceSha256 !== build.staging.nativeSourceSha256) return null;
  if (!build.executable || createHash('sha256').update(await readFile(build.executable)).digest('hex') !== build.executableSha256) return null;
  for (const [relativeSource, digest] of Object.entries(build.staging.nativeSourceFiles ?? {})) {
    const absolute = path.resolve(project, relativeSource);
    if (!absolute.startsWith(project + path.sep)) throw new Error('Invalid integration source path');
    const actual = createHash('sha256').update(await readFile(absolute)).digest('hex');
    if (actual !== digest) return null;
  }
  const registered = evidence.checks.actualModuleRegistry?.observed;
  if (!Array.isArray(registered)) return null;
  return { report: relative, executableSha256: build.executableSha256,
    nativeSourceSha256: build.staging.nativeSourceSha256,
    names: new Set(registered.map(module => module.name)), checks: evidence.checks };
}
const evidence = await integrationEvidence();
const moduleChecks = {
  ModuleZoom: ['zoomRealKeyAndCamera', 'zoomReleaseRestoresCamera', 'zoomBindRespectsOpenScreen'],
  ModuleNoFov: ['noFovActualCamera', 'noFovActualPlayerModifier', 'noFovDisabledRestoresHooks'],
  ModuleFullBright: ['fullBrightActualLightmap', 'fullBrightRealNightVision', 'nightVisionRefreshedByPlayerTicks', 'nightVisionRemoved'],
  ModuleNoBob: ['noBobActualCamera'],
  ModuleNoHurtCam: ['noHurtCamActualCamera'],
};
for (const entry of kotlinFiles) {
  const contents = await readFile(path.join(source, entry.path), 'utf8');
  for (const match of contents.matchAll(/\bobject\s+(\w+)\s*:\s*ClientModule\s*\(/g)) {
    declarations.set(match[1], { symbol: match[1], source: entry.path });
  }
}

let category;
const modules = [];
for (const line of builtin.split('\n')) {
  const heading = line.match(/^\s*\/\/\s*(\w+)\s*$/);
  if (heading) category = heading[1];
  const symbol = line.match(/^\s*(\w+)\s*,?\s*$/)?.[1];
  if (!symbol) continue;
  const declaration = declarations.get(symbol);
  if (!declaration || !category) throw new Error(`Unresolved upstream module: ${symbol}`);
  const rustSource = implementations[symbol];
  if (rustSource) await readFile(path.join(project, rustSource), 'utf8');
  const gameTested = rustSource && evidence?.names.has(symbol === 'ModuleNoFov' ? 'NoFOV' : symbol.slice(6))
    && moduleChecks[symbol]?.every(check => evidence.checks[check]?.passed);
  modules.push({ ...declaration, category,
    rustImplementation: rustSource ? 'native_logic_implemented' : 'not_implemented',
    parityValidation: gameTested ? 'unit_and_native_integration_tested' : rustSource ? 'unit_tested_game_pending' : 'not_run',
    upstreamDifferentialValidation: 'not_run',
    ...(rustSource ? { rustSource } : {}),
    ...(gameTested ? { gameIntegrationEvidence: {
      report: evidence.report, executableSha256: evidence.executableSha256,
      nativeSourceSha256: evidence.nativeSourceSha256 } } : {}) });
}
const eventSource = await readFile(path.join(source,
  'src/main/kotlin/net/ccbluex/liquidbounce/event/EventManager.kt'), 'utf8');
const eventRegistry = eventSource.match(/ALL_EVENT_CLASSES[^=]*= arrayOf\(([\s\S]*?)\n\)/)?.[1];
if (!eventRegistry) throw new Error('Could not locate the upstream event registry.');
const events = [...eventRegistry.matchAll(/(\w+)::class\.java/g)].map(match => match[1]);
const mixins = JSON.parse(await readFile(path.join(source, 'src/main/resources/liquidbounce.mixins.json'), 'utf8'));
const registered = new Set(modules.map(module => module.symbol));
const categories = {};
for (const module of modules) categories[module.category] = (categories[module.category] ?? 0) + 1;

const report = {
  upstreamCommit: 'ae0eb396c07f22383789b28d5912d714c6ca0a86',
  minecraft: '26.3',
  upstreamVersion: '0.43.1',
  moduleRegistry: managerPath,
  registeredModules: modules.length,
  categories,
  kotlinFilesInModuleDirectory: kotlinFiles.length,
  directClientModuleDeclarations: declarations.size,
  unregisteredDeclarations: [...declarations.values()].filter(module => !registered.has(module.symbol)),
  events,
  clientMixins: mixins.client,
  modules,
};
await writeFile(path.join(project, 'research', 'module-parity.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ registeredModules: modules.length, categories, events: events.length,
  clientMixins: mixins.client.length,
  rustModulesNativeIntegrationTested: modules.filter(module => module.gameIntegrationEvidence).length,
  rustModulesImplemented: modules.filter(module => module.rustSource).length,
  rustModulesUpstreamDifferentialTested: 0 }, null, 2));
