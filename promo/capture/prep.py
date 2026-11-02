# Turn raw captures into film assets: scaled sequences + a manifest module.
import os, json, shutil
from PIL import Image
CAP = 'assets/cap'; OUT = 'assets/film'
os.makedirs(OUT, exist_ok=True)
man = {'seq': {}}
def seqdir(name, scale, dur):
    fs = sorted(f for f in os.listdir(f'{CAP}/{name}') if f.endswith('.png'))
    os.makedirs(f'{OUT}/{name}', exist_ok=True)
    for i, f in enumerate(fs):
        im = Image.open(f'{CAP}/{name}/{f}')
        if scale != 1: im = im.resize((round(im.width * scale), round(im.height * scale)), Image.LANCZOS)
        im.save(f'{OUT}/{name}/{i:03d}.png', optimize=False)
    man['seq'][name] = {'n': len(fs), 'dur': dur}
for n, d in [('al_plug', 3.2), ('al_unplug', 3.2), ('al_volume', 2.6), ('al_bright', 2.6), ('al_prayer', 4.5)]: seqdir(n, 0.5, d)
seqdir('eq', 1, 10)
ws = sorted(int(f[:-4]) for f in os.listdir(f'{CAP}/dockw'))
os.makedirs(f'{OUT}/dockw', exist_ok=True)
for w in ws:
    im = Image.open(f'{CAP}/dockw/{w}.png'); im.resize((im.width // 2, im.height // 2), Image.LANCZOS).save(f'{OUT}/dockw/{w}.png')
man['dockw'] = ws
stills = [f for f in os.listdir(CAP) if f.endswith('.png') and f.split('_')[0] in ('pop', 'set', 'dock')]
for f in stills: shutil.copy(f'{CAP}/{f}', f'{OUT}/{f}')
man['stills'] = sorted(s[:-4] for s in stills)
man['slots'] = json.load(open('assets/slots.json'))
open(f'{OUT}/manifest.js', 'w').write('export default ' + json.dumps(man) + ';\n')
print(json.dumps({k: v for k, v in man.items() if k != 'slots'})[:900])
