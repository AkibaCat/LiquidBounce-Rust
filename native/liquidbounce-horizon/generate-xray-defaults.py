from pathlib import Path
import re
ROOT=Path(__file__).resolve().parents[2]
source=(ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/ModuleXRay.kt').read_text(encoding='utf8')
body=source.split('private val defaultBlocks = arrayOf<Block>(',1)[1].split('\n    )',1)[0]
defaults=re.findall(r'^\s*([A-Z][A-Z_]+),?\s*$',body,re.M)
extra=['copper_block','exposed_copper','weathered_copper','oxidized_copper','waxed_copper_block','waxed_exposed_copper','waxed_weathered_copper','waxed_oxidized_copper']
extra += [c+'_shulker_box' for c in ['white','orange','magenta','light_blue','yellow','lime','pink','gray','light_gray','cyan','purple','blue','brown','green','red','black']]
extra += [a+b+'copper_chest' for a in ['','waxed_'] for b in ['','exposed_','weathered_','oxidized_']]
text='// Generated from pinned ModuleXRay defaultBlocks; family initialization stays distinct from reset.\n'
for name,values in [('DEFAULT_BLOCKS',[n.lower() for n in defaults]),('EXTRA_BLOCKS',extra)]:text+=f'pub const {name}: &[&str] = &[\n'+''.join('    "'+n+'",\n' for n in values)+'];\n'
(ROOT/'native/liquidbounce-behaviors/src/xray_defaults.rs').write_text(text,encoding='utf8')
print(len(defaults),'source defaults;',len(extra),'initial family members')
