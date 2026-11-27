"""Command cases for the unchanged common BlockActions source oracle."""
import itertools

def generate(add,new,state,player,at,b,bits,raw,pos,fbits,rng):
 state(11,'oak_door',1,3,opened=0,upper=0)
 state(12,'oak_door',1,3,opened=0,upper=1)
 state(13,'oak_trapdoor',1,10,opened=0)
 def reset():
  new(floor=-7);add('targetnew');add('anew');player(position=(4.3,0,4.3),eye=(2.3,2.1,2.4));add('bridgesent',fbits(-90),fbits(75))
 def hit(where=(0,-1,0),face='UP',valid=True,limit=2**31-1):add('targetsetup',0,bits(4.5),17,fbits(0),pos(where),face,b(valid),limit)
 def setup(latency=0,use=True,start=True,continuing=True,bounds=((4,0,4),(4.6,1.8,4.6))):add('aset',latency,b(use),b(start),b(continuing),raw(bounds[0]),raw(bounds[1]))
 def stack(slot=0,place=True,tool=False,correct=False):add('targetstack',slot,b(tool),b(correct),fbits(8 if correct else 1),2,b(place),'false')
 def place(where=(0,0,0),ready=True,held=True):add('aplace',pos(where),b(ready),fbits(-90)if held else'-',fbits(75)if held else'-')
 # Idle and type transitions preserve or release selections and digging exactly.
 reset();add('aidle');add('aaim');add('atick');at(0,0,0,1);hit((0,0,0));stack(tool=True,correct=True)
 add('amine','0,0,0');add('aaim');add('atick');add('amine','0,0,0');add('atick');add('amine','1,0,0');add('aaim');add('aopen','0,0,0');place();place((1,0,0));place((1,0,0),False,False);add('aopen','0,0,0');add('aopen','1,0,0');add('aidle');add('aidle')
 # Break starts despite a false result, continues with the same block, tracks
 # an empty client shape only once, and honours hit identity/miss and reach+1.
 for valid,where,start,continuing,answer in itertools.product([False,True],[(0,0,0),(1,0,0)],[False,True],[False,True],[0,1]):
  reset();setup(start=start,continuing=continuing);at(0,0,0,1);hit(where,valid=valid);stack(tool=True,correct=True);add('amine','0,0,0');add('atick');add('atick');at(0,0,0,0);add('atick');add('atick');add('aanswer','0,0,0',answer);add('astatus','0,0,0');add('aanswer','0,0,0',1);add('aidle')
 # Open same/other door halves, ordinary opening blocks, wrong hits, closed/
 # open/gone state, failed/successful use and cooldown boundary at ten ticks.
 for state_id,offset,valid,use in itertools.product([0,1,9,10,11,12,13],[-2,-1,0,1,2],[False,True],[False,True]):
  reset();setup(use=use);at(0,0,0,state_id);hit((0,offset,0),valid=valid);add('aopen','0,0,0');add('aaim')
  for tick in range(12):
   add('atick');add('astatus','0,0,0')
   if tick==3:add('aopen','0,0,0')
  add('aanswer','0,0,0',10);add('astatus','0,0,0');add('aidle')
 # Held and planned placement; offhand selection, ready gating, own-box exact
 # intersections, wrong face and miss, empty inventory, failed clicks, and
 # pending replacement creates a distinct last object but leaves old pending.
 for held,slot,ready,use,box,valid in itertools.product([False,True],[None,0,5,40],[False,True],[False,True],['away','overlap','touch'],[False,True]):
  reset();setup(use=use,bounds=((-1,-1,-1),(0,1,1))if box=='touch'else((.1,.1,.1),(.8,1.8,.8))if box=='overlap'else((4,0,4),(4.6,1.8,4.6)));at(0,-1,0,1);hit(valid=valid)
  if slot is not None:stack(slot)
  place(ready=ready,held=held);add('aaim');add('atick');add('atick');add('astatus','0,0,0');place(held=held);add('aaim');add('atick');add('aanswer','0,0,0',1);add('astatus','0,0,0');add('aanswer','0,0,0',0);add('aforget');add('aidle')
 # Retained placement verdict mutation and latency integer division boundaries,
 # replacement in the same map key, no-answer expiry, and level reset retention.
 for latency,kept in itertools.product([-51,-50,-49,-1,0,1,49,50,51,99,100,101,249,250,251],[False,True]):
  reset();setup(latency=latency);at(0,-1,0,1);hit();stack();place();add('atick');add('atick');add('aidle')
  if kept:at(0,0,0,1)
  for tick in range(27):add('atick');add('astatus','0,0,0')
  add('aforget');add('aopen','0,0,0');add('alevel');add('astatus','0,0,0');add('atick')
 # A scripted order contract is intentionally supplied to the Rust adapter.
 # The Kotlin fixture validates this order against its real HashMap keys;
 # production JDK resize/tree-bin identity ordering still needs its adapter.
 for kept_mask in range(8):
  reset();setup(latency=250);stack()
  for x in range(3):
   at(x,-1,0,1);hit((x,-1,0));place((x,0,0));add('atick')
  add('aidle');add('aorder','0,0,0;1,0,0;2,0,0')
  for x in range(3):
   if kept_mask&(1<<x):at(x,0,0,1)
  for tick in range(25):add('atick')
  add('alevel');add('atick')
 # Expiration of open/dug confirmations uses strict > timeout, and a level
 # change preserves ticks, use cooldown and total rejection count.
 for kind,latency,answer_tick in itertools.product(['open','mine'],[0,49,50,100],[10,11,12,13,15,17]):
  reset();setup(latency=latency);hit((0,0,0));at(0,0,0,11 if kind=='open'else 1);stack(tool=True,correct=True)
  add('aopen'if kind=='open'else'amine','0,0,0');add('atick')
  if kind=='mine':at(0,0,0,0);add('atick')
  add('aidle')
  for tick in range(answer_tick):add('atick')
  add('aanswer','0,0,0',1);add('astatus','0,0,0');add('alevel');add('astatus','0,0,0')
 reset();at(0,-1,0,1);hit();stack();place();add('atick');add('aanswer','0,0,0',0)
 at(1,0,0,11);hit((1,0,0));add('aopen','1,0,0');add('atick');add('alevel');add('atick');add('aforget')
