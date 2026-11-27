#!/usr/bin/env python3
"""Static goal adapter parsing; six focused source-selection tests belong to root's combined batch."""
from pathlib import Path
import argparse,ast,subprocess,tempfile
HERE=Path(__file__).resolve().parent
def fixture():
    source=(HERE/'kill-aura-fight-goal.rs').read_text(encoding='utf8')
    pure=source[:source.index('    // NATIVE_FIGHT_GOAL')]
    start=pure.index('    use super::Game;')
    end=pure.index('    use std::cmp::Ordering;')
    pure=pure[:start]+pure[end:]
    # This native Vec3-to-array adapter is not used by the pure selector fixture.
    start=pure.index('    fn point(p: Vec3) -> Point {')
    end=pure.index('    fn distance_squared(',start)
    pure=pure[:start]+pure[end:]
    return(HERE/'kill-aura-fight-goal-smoke.rs').read_text(encoding='utf8').replace('__FIGHT_GOAL_MODEL__',pure)
def main():
    p=argparse.ArgumentParser(description=__doc__);g=p.add_mutually_exclusive_group();g.add_argument('--static-only',action='store_true');g.add_argument('--model-only',action='store_true');a=p.parse_args()
    with tempfile.TemporaryDirectory(prefix='liquidbounce-fight-goal-')as temporary:
        temporary=Path(temporary);f=temporary/'fight-goal-smoke.rs';f.write_text(fixture(),encoding='utf8')
        if not a.model_only:
            ast.parse(Path(__file__).read_text(encoding='utf8'))
            adapter=temporary/'fight-goal.rs';adapter.write_text((HERE/'kill-aura-fight-goal.rs').read_text(encoding='utf8'),encoding='utf8')
            subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',str(f),str(adapter)],check=True)
            text=adapter.read_text(encoding='utf8');assert 'BlockCollisionScan::any_collision'in text and 'Vec3::direction_from_rotation'in text
            assert 'world::read'not in text and 'world::write'not in text and 'world::with_scope'not in text
            print('FightBot goal static checks passed; owned adapter and pure fixture parse; no production/shared/staging writes')
        if not a.static_only:
            exe=temporary/'fight-goal-smoke.exe';subprocess.run(['rustc','--edition=2024','--test',str(f),'-o',str(exe)],check=True);subprocess.run([str(exe),'fight_goal_smoke','--nocapture'],check=True)
        else:print('Six focused Rust tests deferred to combined module batch')
if __name__=='__main__':main()
