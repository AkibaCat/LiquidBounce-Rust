"""Native PotionFX texture assets, nine pinned images and independent quad renderer."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'breadcrumbs-render.rs').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]
ASSETS=[(n,'potion_fx/main/'+n.lower()+'.png')for n in ['Dashed','Solid','Runes','Atlas']]+[(n,'potion_fx/secondary/'+n.lower()+'.png')for n in ['Cracked','Neuron','Hexagon','Stardust']]+[('Glow','particles/glow.png')]
def source():return(HERE/'potion-fx-render.rs').read_text(encoding='utf8')
def textures():
 root=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce'
 return 'const PNGS:[(&str,&str);9]=[\n'+''.join('("'+n+'","'+base64.b64encode((root/p).read_bytes()).decode()+'"),\n'for n,p in ASSETS)+'];\n'
