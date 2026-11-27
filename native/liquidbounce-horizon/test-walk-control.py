#!/usr/bin/env python3
"""Unchanged original walking support state, paths, getters and world queries."""
from pathlib import Path
import argparse,hashlib,itertools,json,os,random,re,runpy,shutil,struct,subprocess,time
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
AGENT=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent'
def bits(v):return struct.unpack('<Q',struct.pack('<d',v))[0]
def fbits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def raw(v):return ','.join(str(bits(float(x)))for x in v)
def pos(v):return ','.join(map(str,v))
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--smoke',action='store_true');args=p.parse_args();out=(ROOT/args.out).resolve();out.relative_to(ROOT/'build')
 if out.exists():raise SystemExit('Use a fresh evidence directory')
 out.mkdir(parents=True);commands=[];records=[];rng=random.Random(0x26_03_55)
 def add(*a):commands.append(' '.join(map(str,a)))
 def b(v):return str(v).lower()
 for ident,shape in [(0,'E'),(1,'F'),(2,raw([0,0,0,1,.5,1])),(3,raw([0,0,0,.125,1,1]))]:add('shapeDef',ident,shape)
 def state(i,name,shape=1,flags=0,fluid=0,opened=-1,upper=-1,facing=-1,friction=.6):add('stateDef',i,name,flags,fluid,opened,upper,facing,fbits(0),shape,shape,-1,-1,fbits(friction),fbits(1),fbits(1),b(shape==0))
 state(0,'air',0);state(1,'stone');state(2,'void_air',0);state(3,'water',0,fluid=1);state(4,'ladder',3,16,facing=5);state(5,'oak_stairs',2,128,upper=0);state(6,'oak_stairs',1,128,upper=1);state(7,'farmland',2,64);state(8,'ice',friction=.98);state(9,'oak_door',1,3,opened=0);state(10,'oak_door',0,3,opened=1)
 def new(legacy=False,floor=-1):add('new','false',bits(struct.unpack('<f',struct.pack('<f',.6))[0]),bits(struct.unpack('<f',struct.pack('<f',1.8))[0]),-16,64,bits(-100.),bits(100.),b(legacy),floor,1);add('cnew')
 def at(x,y,z,i):add('set',f'{x},{y},{z}',i)
 def player(tick=0,position=(.5,0,.5),delta=(0,0,0),speed=.1,strength=.42,boost=0,flags=1,food=20,air=300,max_air=300,eye=None,support=None,below=(0,-1,0),delay=0,yaw=-90):add('player',tick,raw(position),raw(delta),bits(speed),bits(strength),fbits(boost),flags,food,air,max_air,raw(eye or(position[0],position[1]+1.62,position[2])),pos(support)if support else'-',pos(below),delay,fbits(yaw))
 def step(i,fr=(0,0,0),to=(1,0,0),move='TRAVERSE',feet=None,breaks=(),places=(),opens=(),flags=0,runup=(-1.5,0,.5)):
  add('step',i,pos(fr),pos(to),move,raw(feet or(to[0]+.5,to[1],to[2]+.5)),';'.join(map(pos,breaks))or'-',';'.join(map(pos,places))or'-',flags,';'.join(map(pos,opens))or'-',raw(runup))
 if args.smoke:
  runpy.run_path(str(HERE/'walk-control-source-fixture/capture_smoke.py'))['generate'](add,new,player,b,bits,raw,fbits)
 else:
  # Temporal original states and boundaries, including same-tick setback floods.
  new();step(0);step(1,to=(1,1,0),move='ASCEND');add('route',raw((.5,0,.5)),'0,1')
  for i in range(75):add('act','1,0,0');add('watch',bits(3.))
  add('unact');add('act','1,0,0');add('excuse')
  for i in range(150):
   tick=i//3 if i<60 else i*3;point=(.5+(i%7)*.21,(i%3)*.1,.5+(i%5)*.17)
   add('refuse',raw(point),tick)
  for point,tick in [((4.,0.,0.),10),((4.2,0.,0.),11),((5.5,0.,0.),12),((5.5,0.,0.),72),((7.,0.,0.),73),((-2.,-1.,-3.),2**63-1),((-2.1,-1.,-3.),-2**63)]:add('refuse',raw(point),tick)
  add('goal',raw((0,0,0)))
  for i in range(605):
   if i in [0,1,300]:add('stuck',f'{i%2},0,0')
   add('goal',raw((0,0,0)))
  add('stuck','1,0,0');add('goal',raw((8,0,0)));add('goal',raw((8.0001,0,0)));add('stuck','2,0,0');add('forget');add('stuck','2,0,0');add('goal',raw((-8.1,0,0)))
  # Stride retained measurements, skipped/same ticks, attributes, environmental gates.
  for legacy in [False,True]:
   new(legacy);x=.5;carried=.16
   for tick in range(1,181):
    speed=.1 if tick%23<16 else .13;flags=[1,3,1,1,1,1,1,1,5,9,17,33,65,0][tick%14]
    x+=carried+speed*.98*([.1,.25,.5,1.,2.,8.][tick%6]);player(tick if tick%17 else tick-1,(x,0,.5),(carried,0,0),speed=speed,flags=flags,below=(0,-1,0),boost=.2 if tick%9==0 else 0)
    at(0,-1,0,8 if tick%11==0 else 1);add('stride');add('abilities',fbits(.1),fbits(.42),fbits(0))
  # Step/route geometry against arbitrary turns, ladders, blocks and run-ups.
  for scene in range(32):
   new();coords=[(0,0,0)];ids=[]
   for i in range(12):
    fr=coords[-1];to=(fr[0]+rng.choice([-1,0,1]),fr[1]+rng.choice([-1,0,0,0,1]),fr[2]+rng.choice([-1,0,1]));coords.append(to);ids.append(str(i))
    move=rng.choice(['TRAVERSE','DIAGONAL','ASCEND','FALL','PARKOUR','PILLAR','DOWNWARD','SWIM','CLIMB_UP','CLIMB_DOWN']);flags=rng.choice([0,0,0,1,2,4,8]);places=[(to[0],to[1]-1,to[2])]if scene%5==0 else[]
    if scene%11==0:move='ASCEND';places=[(to[0],to[1]-2,to[2]),(to[0],to[1]-1,to[2])]
    step(i,fr,to,move,flags=flags,places=places,breaks=[(to[0],to[1]+1,to[2])]if scene%7==0 else[])
    q=(to[0]+rng.uniform(-1,1),to[1]+rng.uniform(-1.4,1.4),to[2]+rng.uniform(-1,1))
    add('stepfacts',i,raw(q),raw((rng.uniform(-.5,.5),0,rng.uniform(-.5,.5))),fbits(rng.uniform(-720,720)),b(i%2==0),b(i%3==0))
    add('center',raw(q),raw((to[0]+.5,to[1],to[2]+.5)),fbits(rng.uniform(-720,720)))
   add('route',raw((.5,0,.5)),','.join(ids))
   for i in range(12):
    q=(coords[i][0]+rng.uniform(-2,2),coords[i][1]+rng.uniform(-1.2,1.2),coords[i][2]+rng.uniform(-2,2));distance=rng.uniform(.01,15)
    add('off',i,raw(q));add('join',raw(q));add('look',i,raw((coords[min(12,i+2)][0]+.5,coords[min(12,i+2)][1],coords[min(12,i+2)][2]+.5)),raw(q),bits(distance));add('point',i,raw(q),bits(distance));add('turn',i,raw(q),bits(distance));add('bridge',i);add('run',i)
    if i<11:add('straight',i,i+1)
   add('off',12,raw((0,0,0)))
  # Plain and rising bridge runs and end turns, with line pursuit projection.
  for turn in [-1,0,1]:
   for rising in [False,True]:
    new()
    for i in range(6):
     y=i if rising else 0;ty=i+1 if rising else 0;step(i,(i,y,0),(i+1,ty,0),'ASCEND'if rising else'TRAVERSE',places=[(i+1,ty-2,0),(i+1,ty-1,0)]if rising else[(i+1,-1,0)])
    step(6,(6,6 if rising else 0,0),(7,6 if rising else 0,turn));add('route',raw((.5,0,.5)),'0,1,2,3,4,5,6')
    for i in range(7):add('bridge',i);add('run',i)
  for _ in range(200):
   fr=(rng.uniform(-10,10),rng.uniform(-2,2),rng.uniform(-10,10));to=(rng.uniform(-10,10),rng.uniform(-2,2),rng.uniform(-10,10));q=(rng.uniform(-10,10),0,rng.uniform(-10,10));add('line',raw(fr),raw(to),raw(q))
  add('line',raw((1,2,3)),raw((1,2,3)),raw((4,5,6)))
  # Overshooting retains reversal count; callback ordering and airborne fallback.
  new();step(0);step(1,flags=1)
  for tick in range(100):
   player(tick,delta=(rng.choice([-.4,0,.3,.4,1.]),0,0),flags=tick%2)
   add('overshoot',tick%2,raw((.5,0,.5)),raw((rng.choice([-1.,.4,2.]),0,.5)),raw((2.,0,.5))if tick%3 else'-')
  # Raw completion world queries, unconfirmed placements, live stairs and stable inputs.
  for kind,flags in itertools.product(range(9),[0,1,3,16,32]):
   new();step(0,flags=8 if kind==1 else 0,breaks=[(1,1,0)]if kind==2 else[],places=[(1,-1,0)]if kind in [3,4]else[],opens=[(1,0,0)]if kind==5 else[]);step(1,(1,0,0),(2,0,0));add('route',raw((.5,0,.5)),'0,1')
   player(position=(1.7,0,.5),flags=flags,support=(1,-1,0))
   at(1,1,0,1 if kind==2 else 0);at(1,-1,0,0 if kind==3 else 1);at(1,0,0,9 if kind==5 else 5 if kind==6 else 6 if kind==7 else 0)
   if kind==4:add('confirm','1,-1,0','true')
   add('reached',0,1);add('skip',0);add('wants',0);add('stairs',0)
  # Swimming breathing hysteresis, air search stopping at first empty-fluid block.
  for mode in range(7):
   new();step(0,move='SWIM',feet=(1.5,.2,.5));
   for y in range(1,7):at(0,y,0,3 if y<mode+1 else 1 if mode%2 else 0)
   for air,flags in [(300,17),(89,17),(95,17),(299,17),(300,17),(40,1),(40,145)]:
    player(position=(.5,0,.5),flags=flags,air=air,eye=(.5,1.62,.5));add('breathes');add('shortair');add('swim',0,fbits(-90))
  # Terrain live climbing and sprint hopping gates, all sample/query order retained.
  for mode in range(7):
   new();step(0,move='CLIMB_UP',to=(0,1,0));at(0,0,0,4 if mode%2 else 0);at(0,1,0,4 if mode%2 else 0)
   if mode==2:at(1,0,0,1)
   player(flags=1);add('climb',0,fbits(-90))
  for flags,food,distance,yaw,scene in itertools.product([0,1,3],[10,11],[3.99,4.,8.],[-90,-75,-74.9,90],['clear','farmland','roof']):
   new();player(flags=flags,food=food,yaw=yaw)
   if scene=='farmland':at(2,-1,0,7)
   if scene=='roof':at(2,2,0,1)
   add('hop',raw((.5,0,.5)),raw((.5+distance,0,.5)));add('clearhop')
  # Prediction's exact host protocol, including early success, 20-tick failure,
  # dive/no-target bypass, stable/water jump updates, and floor braking retry.
  def script(frames):return ';'.join(raw(p)+'^'+raw(d)+'^'+str(f)for p,d,f in frames)or'-'
  for success,flags,keys in itertools.product([0,1,8,19,99],[0,1,2,8,9],[0,64,66,68,96]):
   new();step(0,to=(3,1,0),move='ASCEND');player(position=(.5,0,.5),delta=(.2,0,0),flags=1)
   frames=[((.5 if tick<success else 1.,0,.5),(.1,0,0),[0,1,16,32][tick%4])for tick in range(24)]
   add('script','true',script(frames),script(frames));add('predict',0,keys,fbits(-90),raw((3.5,1,.5))if success!=19 else'-',flags)
  for stops,keys,sneaks in itertools.product([0,1,3,4,5,99],[0,64,66],[False,True]):
   new();step(0);player(position=(.9,0,.5),delta=(.12,0,0),flags=1)
   moving=[((1.31 if stops==99 else .95+tick*.04,0,.5),(0. if tick>=stops else .12,0,0),1)for tick in range(7)]
   still=[((.95,0,.5),(0.,0,0),1)]
   add('script','true',script(moving),script(still));add('floor',0,keys,fbits(-90),b(sneaks))
  # The real common PlayerMotion fallback over the actual controlled world;
  # both mechanics, blocked/no-progress, rise, stair, grounded/airborne cases.
  for legacy,scene,keys in itertools.product([False,True],['flat','wall','rise','stairs','ice','air'],[0,64,68,66]):
   new(legacy);step(0,to=(3,1 if scene in ['rise','stairs']else 0,0),move='ASCEND'if scene in ['rise','stairs']else'TRAVERSE')
   if scene=='wall':
    for z in [-1,0,1]:at(1,0,z,1);at(1,1,z,1)
   if scene=='rise':at(1,0,0,1);at(2,0,0,1)
   if scene=='stairs':at(1,0,0,5);at(2,0,0,1)
   if scene=='ice':
    for x in range(5):at(x,-1,0,8)
   player(position=(.5,1.,.5)if scene=='air'else(.5,0,.5),delta=(.1,-.1,0),flags=0 if scene=='air'else 1,support=None if scene=='air'else(0,-1,0))
   add('script','false','-','-');add('predict',0,keys,fbits(-90),raw((3.5,1 if scene in ['rise','stairs']else 0,.5)),9)
  for forward,strafe,frame,angle in itertools.product([-1,0,1],[-1,0,1],[-360.,-90.,0.,45.,180.,720.],[None,-180.,-135.,-90.,-45.,0.,45.,90.,135.,180.,1.e30]):add('correct',forward,strafe,fbits(frame),'-'if angle is None else fbits(angle))
  for _ in range(700):add('correct',rng.randrange(-1,2),rng.randrange(-1,2),fbits(rng.uniform(-1000,1000)),fbits(rng.uniform(-1000,1000)))
  # Public Leap.tick and the NavigationContext.leap wrapper retain lifecycle,
  # exact original live reset reads, measured traits and movement correction.
  for legacy,scene in itertools.product([False,True],['gap','flat','wall','air','runup']):
   new(legacy,floor=-7);step(0,to=(3,0,0),move='PARKOUR',flags=8,runup=(-1.5,0,.5)if scene=='runup'else(.5,0,.5))
   for x in range(-3,1):at(x,-1,0,1)
   at(3,-1,0,1)
   if scene=='flat':
    for x in [1,2]:at(x,-1,0,1)
   if scene=='wall':at(1,0,0,1);at(1,1,0,1)
   for tick in range(8):
    player(tick,position=(.5,.7,.5)if scene=='air'else(.5,0,.5),delta=(.15,.1,0)if scene=='air'else(0,0,0),flags=0 if scene=='air'else 1,yaw=90 if tick<3 else -90,support=None if scene=='air'else(0,-1,0),speed=.13 if tick>5 else .1)
    add('leaping',0,fbits(-90 if tick%2 else 0))
  # Original view rounding, turning caps/easing and bridging cadence/selection.
  methods=['AUTO','NINJA','GODBRIDGE','MOONWALK','BREEZILY','ONE_STACK','SWIFT_SNEAK','TELLY']
  for method,block,along,turn in itertools.product(methods,[-15,-1,0,1,6,7,8,12,14,15,19,24,30,2**31-1],[.0,.349999999,.35,.5,.6,.600000001,1.],[-1,0,1]):add('bridgecadence',method,block,bits(along),turn)
  for _ in range(1200):
   vector=[rng.uniform(-1e8,1e8)for _ in range(3)];add('rotation',raw(vector));add('viewvector',fbits(rng.uniform(-1e6,1e6)),fbits(rng.uniform(-1e6,1e6)))
  for vector in itertools.product([-1.,-0.,0.,1.],repeat=3):add('rotation',raw(vector))
  for _ in range(1800):
   add('turning',*[fbits(rng.uniform(-1000,1000))for _ in range(4)],rng.choice(['instant','linear','eased']),fbits(rng.choice([0.,.1,1.5,40.,90.,180.])),fbits(rng.choice([0.,.1,1.5,28.,63.,180.])),fbits(rng.choice([0.,.45,.6,1.,2.])),fbits(rng.choice([0.,1.5,10.,100.])),b(rng.choice([False,True])),rng.choice(['MOVEMENT','AIM','VIEW','STEER']))
  for rising,turn,length in itertools.product([False,True],[-1,0,1],[1,3,4,9,10,14,15,18]):
   new()
   for i in range(length):
    y=i if rising else 0;ty=i+1 if rising else 0;step(i,(i,y,0),(i+1,ty,0),'ASCEND'if rising else'TRAVERSE',places=[(i+1,ty-2,0),(i+1,ty-1,0)]if rising else[(i+1,-1,0)])
   step(length,(length,length if rising else 0,0),(length+1,length if rising else 0,turn));add('route',raw((.5,0,.5)),','.join(map(str,range(length+1))))
   for method,risk,sneak,sprint in itertools.product(methods,[False,True],[.3,.6999999,.7,1.],[False,True]):add('bridgepick',method,0,b(risk),bits(sneak),b(sprint))
  # Telly and Crossing run against the unmodified controllers, real raw block
  # checks and scripted exact Body frames. Actions.place itself remains the real
  # original state container; this does not exercise world-facing action ticks.
  for method,scene in itertools.product(methods,['level','rise','turn','finished','water','air']):
   new(floor=-7);rising=scene=='rise'
   for i in range(12):
    y=i if rising else 0;ty=i+1 if rising else 0;step(i,(i,y,0),(i+1,ty,0),'ASCEND'if rising else'TRAVERSE',places=[(i+1,ty-2,0),(i+1,ty-1,0)]if rising else[(i+1,-1,0)])
    if scene=='finished':
     at(i+1,ty-1,0,1)
     if rising:at(i+1,ty-2,0,1)
   step(12,(12,12 if rising else 0,0),(13,12 if rising else 0,1 if scene=='turn'else 0));add('route',raw((.5,0,.5)),','.join(map(str,range(13))));add('bridgenew',fbits(-90),fbits(0),method,b(scene in ['turn','finished']),bits(.7 if method=='SWIFT_SNEAK'else .3))
   for tick in range(28):
    index=min(11,tick//3);x=index+.5+[.0,.4,.7][tick%3];y=index if rising else .8 if scene=='air'else 0;flags=16 if scene=='water'else 0 if scene=='air'or tick%5==2 else 1
    player(tick,position=(x,y,.5),delta=([0.,.09,.11,.3][tick%4],-.1 if tick%3==0 else .1,0),flags=flags,yaw=-90)
    moving=[((x+.15,y,.5),(.1,0,0),1 if tick%4 else 257),((x+.2,y,.5),(0.,0,0),1)]
    still=[((x,y-.02,.5),(0.,0,0),0),((x,y,.5),(0.,0,0),1 if tick%3 else 0)]
    add('script','true',script(moving),script(still))
    if tick%3==0:add('placed',pos((index,-1,0)),tick)
    if tick%6==0:
     at(index+1,y-1 if rising else-1,0,1)
     add('confirm',pos((index+1,y-1 if rising else-1,0)),b(tick%12==0))
    add('takes',index)
    add('cross',index,fbits(-90),64 if tick%2 else 0)
    if not rising:add('telly',index,fbits(-90),0,fbits(75))
   add('reject',1);add('takes',0)
   add('reject',2);add('takes',0);add('crossclear');add('takes',12)
  faces=['DOWN','UP','NORTH','SOUTH','WEST','EAST'];units=[(0,-1,0),(0,1,0),(0,0,-1),(0,0,1),(-1,0,0),(1,0,0)]
  for face,seed in itertools.product(faces,[-2**63,-1000,-1,0,1,17,2**31-1,2**63-1]):add('clickpoint','-17,3,65',face,seed)
  for _ in range(900):add('clickpoint',pos((rng.randrange(-10000,10000),rng.randrange(-100,100),rng.randrange(-10000,10000))),rng.choice(faces),rng.randrange(-2**63,2**63))
  # Real original BlockTargets inventory/tool helpers through live stack objects.
  new();add('targetnew')
  for scene in range(240):
   add('targetsetup',scene%9,bits(4.5),scene,fbits(0),'0,-1,0','UP','true',2**31-1)
   for slot in [*range(9),40]:add('targetstack',slot,b(rng.choice([False,True])),b(rng.choice([False,False,True])),fbits(rng.choice([0.,1.,1.01,2.,8.,16.,100.])),rng.choice([0,1,2,3,5,50000]),b(rng.choice([False,False,True])),b(rng.choice([False,True])))
   add('targetdig',1);add('targetslot');add('targetheld','0,0,0',fbits(-90),fbits(80))
  # Raw geometry, actual original seeded clickPoint, six neighbor directions,
  # exact view rounding, visibility selection and view-step margins.
  for direction,nearest,scene in itertools.product(range(6),[False,True],['clear','under','outward','far','blocked','miss']):
   new(floor=-7);add('targetnew');unit=units[direction];against=unit;face=faces[direction^1]
   at(*against,1);at(0,0,0,1 if scene=='blocked'else 0)
   eye=(2.3,2.1,2.4)if scene not in ['outward','far']else tuple(-10.*q for q in unit)if scene=='outward'else(200.,200.,200.)
   if scene=='under':eye=(.5,2.1,.5)
   player(position=(0.,2.,0.)if scene=='under'else(3.,2.,3.),eye=eye)
   add('bridgesent',fbits(rng.uniform(-360,360)),fbits(rng.uniform(-90,90)))
   add('targetsetup',0,bits(4.5),rng.randrange(-2**63,2**63),fbits(0),pos(against),face,b(scene!='miss'),2**31-1)
   add('targetplan','0,0,0',-1 if nearest else 4,b(nearest))
   add('targetclicks',raw(eye),fbits(-90),fbits(80),'0,0,0')
  for shape,step_size,hit,limit in itertools.product([0,1,5],[0.,.01,1.,100.],[False,True],[0,1,124,125,128,100000]):
   new(floor=-7);add('targetnew');at(0,0,0,shape);player(eye=(2.1,1.6,2.2));add('bridgesent',fbits(-130),fbits(30))
   add('targetsetup',0,bits(4.5),17,fbits(step_size),'0,0,0','WEST',b(hit),limit);add('targetaim','0,0,0')
  runpy.run_path(str(HERE/'walk-control-source-fixture/actions_cases.py'))['generate'](add,new,state,player,at,b,bits,raw,pos,fbits,rng)
  runpy.run_path(str(HERE/'walk-control-source-fixture/shortcut_cases.py'))['generate'](add,new,state,player,at,b,bits,raw,pos,fbits,rng,step)
  runpy.run_path(str(HERE/'walk-control-source-fixture/follower_cases.py'))['generate'](add,new,state,player,at,b,bits,raw,pos,fbits,rng,step)
  runpy.run_path(str(HERE/'walk-control-source-fixture/capture_cases.py'))['generate'](add,new,state,player,at,b,bits,raw,pos,fbits,rng,step)
 (out/'commands.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
 original=sorted([*(AGENT/'navigation/walk').glob('*.kt'),AGENT/'navigation/Bridging.kt',AGENT/'navigation/Turning.kt',AGENT/'navigation/BlockSight.kt',AGENT/'math/ViewRotation.kt',AGENT/'math/VectorExtensions.kt',AGENT/'pathfinder/walk/StraightWalk.kt',AGENT/'navigation/AbilityCapture.kt',AGENT/'pathfinder/walk/BreakTimes.kt'])
 for i,q in enumerate(original):shutil.copyfile(q,out/f'{i:02}_{q.name}')
 base=(HERE/'navigation-source-fixture/Terrain.kt').read_text();imports,code=(HERE/'walk-control-source-fixture/WalkControl.kt').read_text().split('// fixture code\n',1)
 for name in ['Targets.kt','Actions.kt','Follower.kt','Capture.kt']:
  more_imports,more_code=(HERE/'walk-control-source-fixture'/name).read_text().split('// fixture code\n',1);imports+='\n'+more_imports;code+='\n'+more_code
 base=base.replace('package fixture','package fixture\n'+imports).replace('BlockKind.FIRE)','BlockKind.FIRE,BlockKind.FARMLAND,BlockKind.STAIRS)').replace('override fun canBeReplaced()=isAir','override fun canBeReplaced()=replaced[id]?:isAir').replace('states[id]=state','states[id]=state;replaced[id]=a[16].toBoolean()').replace('.6f,1f,1f,Float.fromBits','Float.fromBits(a[13].toUInt().toInt()),Float.fromBits(a[14].toUInt().toInt()),Float.fromBits(a[15].toUInt().toInt()),Float.fromBits').replace('BlockStateProperties.DOUBLE_BLOCK_HALF->','BlockStateProperties.HALF->when(upper){0->Half.BOTTOM;1->Half.TOP;else->null}\n  BlockStateProperties.DOUBLE_BLOCK_HALF->').replace('fun main(){','private val replaced=HashMap<Int,Boolean>()\n'+code+'\nfun main(){').replace('else->error(command)','else->result=extra(a,world,terrain)')
 base=base.replace('override fun getDestroySpeed(level:BlockGetter,pos:BlockPos)=0f','override fun getDestroySpeed(level:BlockGetter,pos:BlockPos):Float{events+="hard:$id";return captureHardness[id]?.first?:0f}').replace('override fun requiresCorrectToolForDrops()=false','override fun requiresCorrectToolForDrops():Boolean{events+="requires:$id";return captureHardness[id]?.second?:false}')
 (out/'Fixture.kt').write_text(base,encoding='utf-8')
 fixture=(HERE/'navigation-source-fixture/terrain.rs').read_text();extra='\n'.join((HERE/'walk-control-source-fixture'/name).read_text()for name in ['walk_control.rs','targets.rs','actions.rs','follower.rs','capture.rs'])
 fixture=fixture.replace('struct Registry {','struct Registry {\n walk_facts:HashMap<String,(f32,f32,f32,bool)>,\n capture_hardness:HashMap<i32,(f32,bool)>,').replace('let mut r = registry.borrow_mut();','let mut r = registry.borrow_mut();\n r.walk_facts.insert(block.clone(),(f32::from_bits(a[13].parse().unwrap()),f32::from_bits(a[14].parse().unwrap()),f32::from_bits(a[15].parse().unwrap()),a[16].parse().unwrap()));').replace('#[test]',extra+'\n#[test]',1).replace('let mut results = Vec::new();','let mut walk_fixture=None;let mut results = Vec::new();').replace('_ => panic!("{command}"),','_ => result=extra(&a,t,&mut walk_fixture),').replace('unchanged_agent_terrain_shapes_stands_opening_and_invalidation','unchanged_agent_walk_control_state_paths_player_getters_and_world_queries')
 fixture=fixture.replace('type Events = Rc<RefCell<Vec<String>>>;', 'type Events = Arc<std::sync::Mutex<Vec<String>>>;').replace('let events: Events = Rc::new(RefCell::new(Vec::new()));','let events: Events = Arc::new(std::sync::Mutex::new(Vec::new()));')
 fixture=re.sub(r'(\bevents\s*)\.borrow_mut\(\)',r'\1.lock().unwrap()',fixture);fixture=re.sub(r'(\bevents\s*)\.borrow\(\)',r'\1.lock().unwrap()',fixture)
 if args.smoke:fixture+='\n'+(HERE/'walk-control-source-fixture/planning_smoke.rs').read_text()
 (out/'walk_control_oracle.rs').write_text(fixture,encoding='utf-8')
 kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar';slf4j=next(kotlin.glob('slf4j-api-*.jar'));cp=os.pathsep.join(map(str,[next(kotlin.glob('kotlin-stdlib-*.jar')),binary,fastutil,slf4j]))
 def run(name,argv,env=None):
  began=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');records.append(dict(name=name,exitCode=r.returncode,seconds=round(time.monotonic()-began,3)));(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print((r.stdout+r.stderr)[-14000:]if r.returncode else r.stdout+r.stderr,end='');return r.returncode
 result=run('compile',[str(java),'-cp',str(kotlin/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-jvm-target','25','-Xfriend-paths='+str(binary),'-opt-in=net.ccbluex.agent.InternalAgentApi','-classpath',cp,'-d',str(out/'classes'),*[str(q)for q in out.glob('*.kt')]])
 if not result:result=run('oracle',[str(java),'-Xmx1g','-Dcommands='+str(out/'commands.txt'),'-Dresults='+str(out/'results.txt'),'-cp',os.pathsep.join([str(out/'classes'),cp]),'fixture.FixtureKt'])
 nav=ROOT/'native/liquidbounce-behaviors/src/navigation';shutil.copytree(nav,out/'navigation')
 with(out/'navigation/mod.rs').open('a',encoding='utf-8')as stream:stream.write('\npub mod walk_control;\n')
 snapshot={str(q.relative_to(out/'navigation')):hashlib.sha256(q.read_bytes()).hexdigest()for q in(out/'navigation').rglob('*')if q.is_file()}
 (out/'lib.rs').write_text('#![allow(dead_code)]\n#[path="navigation/mod.rs"]pub mod navigation;#[cfg(test)]mod walk_control_oracle;\n',encoding='utf-8');(out/'Cargo.toml').write_text('[package]\nname="liquidbounce-walk-control-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf-8')
 if not result:
  env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/walk-control-target');result=run('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],env)
  if not result and not args.smoke:result=run('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'],env)
 files=[*original,binary,fastutil,slf4j,Path(__file__),*sorted((HERE/'walk-control-source-fixture').glob('*')),HERE/'navigation-source-fixture/Terrain.kt',HERE/'navigation-source-fixture/terrain.rs']
 report=dict(passed=len(records)==(3 if args.smoke else 4) and all(r['exitCode']==0 for r in records),commands=records,steps=len(commands),sourceSha256={str(q.relative_to(ROOT))if q.is_relative_to(ROOT)else str(q):hashlib.sha256(q.read_bytes()).hexdigest()for q in files if q.is_file()},compiledNavigationSnapshotSha256=snapshot,unchangedOriginalKotlin=all(q.read_bytes()==(out/f'{i:02}_{q.name}').read_bytes()for i,q in enumerate(original)),scope='walking support source state, path geometry, Stride, Swimming, Hops, Climbing, StepCheck, Overshoot, Prediction/floor loops, public Leap lifecycle, NavigationContext.leap, movement correction, ViewRotation, Turning factories, Bridging selection/cadence, Telly/Crossing controller sequences, BlockTargets inventory/tools/placement geometry, seeded clickPoint and complete block seenPoint grid/step sampling; raw numeric bits, player getters and ordered actual world queries',boundaries=['Follower/BlockActions/Planning orchestration not implemented here','Telly/Crossing Actions.place state compared; real world-facing BlockActions.tick not exercised','Crossing target-nudge path not exercised by bridge sequences; BlockTargets tested independently','exact simulation and ray-clip hosts are scripted to verify controller protocol; actual 26.3 SimulatedPlayer native adapter not implemented here','actual native world binding and full-client differential not_run'])
 report['scope']+=', full BlockActions mine/open/place/aim/tick/answer/expiry lifecycle, placement identity/verdict updates, logger/listener ordering and integer latency/cooldown boundaries'
 report['scope']+=', complete StraightWalk/Shortcut with real Terrain caches, side-drop/water/known/border/ground-level queries and farthest-first plain-route candidate selection'
 report['scope']+=', full Follower orchestration/lifecycle with real BlockActions and Crossing binding, route/vector provenance, retained placement identities, stepReached callbacks, off-path/progression, pending break/place/open/confirmation guards, external floor bridge lines, fall/stall handling, swimming/climbing/leaping and plain movement; original Crossing start/fallback/hop logger callbacks'
 report['boundaries'][:2]=['Planning orchestration and AutoWalk runtime registration not implemented here','BlockActions pending expiration order is an explicit fixture-supplied PlacementOrder contract checked against Kotlin HashMap; production JDK bucket/resize/tree identity adapter remains missing']
 if args.smoke:
  report['mode']='targeted_smoke'
  report['scope']='three AbilityCapture/BreakTimes usability scenes against unchanged original sources: option gates, offhand/hotbar counts, copied tools, mining hardness, retained break policy and copied sets; Rust Planning constructor/stuck/drop/reset lifecycle smoke'
  report['boundaries']=['Planning search acceptance and native world binding are outside this smoke','prior Follower18 evidence is retained separately; this run does not replace its 75,714 cases','AutoWalk registration and full-client differential not_run']
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(passed=report['passed'],steps=len(commands))));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
