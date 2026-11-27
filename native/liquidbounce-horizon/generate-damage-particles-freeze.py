from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 target=HERE/'damage-particles-freeze.json'
 if target.exists():raise SystemExit('Refusing to overwrite historical freeze')
 files=[ROOT/'native/liquidbounce-behaviors/src/damage_particles.rs',HERE/'engine/src/config/damage_particles.rs',HERE/'engine/src/damage_particles_runtime.rs']
 files += [HERE/name for name in ['damage-particles-host.rs','damage-particles-game.rs','damage-particles-capture.rs','damage-particles-dragon.rs','damage-particles-patch.py','damage-particles-smoke.rs','test-damage-particles.py','integrate-damage-particles.py','update-damage-particles-locales.py','damage-particles-integration.md','generate-damage-particles-freeze.py']]
 files += [ROOT/'build/damage-particles-smoke-02/report.json',ROOT/'build/damage-particles-smoke-02/test.log']
 focused=json.loads((ROOT/'build/damage-particles-smoke-02/report.json').read_text(encoding='utf8'));assert focused['passed']
 report={'module':'DamageParticles','date':'2026-11-26','pinned':{'liquidbounce':'ae0eb396c07f22383789b28d5912d714c6ca0a86','horizon':'9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'},'focused':focused,'files':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files}}
 target.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(hashlib.sha256(target.read_bytes()).hexdigest())
if __name__=='__main__':main()
