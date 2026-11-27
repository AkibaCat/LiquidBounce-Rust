from pathlib import Path
import hashlib, json
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

def main():
    target = HERE / 'tracers-freeze-03.json'
    assert not target.exists()
    data = json.loads((HERE / 'tracers-freeze-02.json').read_text(encoding='utf8'))
    data['revision'] = 3
    data['revisionNote'] = 'Corrected real Mob UUID and is_baby APIs after native check 116-01; matched mannequin grouping; no repeated smoke.'
    for name in ['tracers-game.rs', 'tracers-revision-03.md', 'generate-tracers-freeze-03.py']:
        p = HERE / name
        data['files'][p.relative_to(ROOT).as_posix()] = hashlib.sha256(p.read_bytes()).hexdigest()
    target.write_text(json.dumps(data, indent=2) + '\n', encoding='utf8')
    print(hashlib.sha256(target.read_bytes()).hexdigest())

if __name__ == '__main__':
    main()
