"""Actual unchanged Follower lifecycle, advance, movement and action cases."""
import itertools

def generate(add,new,state,player,at,b,bits,raw,pos,fbits,rng,step):
 def reset():new();add('targetnew');add('anew');add('bridgenew',fbits(0),fbits(0),'AUTO','false',bits(.3));add('fnew')
 def route(ids):add('route',raw((.5,0,.5)),','.join(map(str,ids))or'-')
 def props():
  add('foff');add('fnext');add('fruns');add('fpoint',raw((.5,0,.5)),bits(2.));add('flook',raw((2.5,0,.5)),raw((.5,0,.5)),bits(3.));add('fturn',bits(5.));add('fhead',fbits(.5))
 def script(x=.5,y=0,z=.5,progress=True):
  frame=raw((x+(.3 if progress else 0),y,z))+'^'+raw((.0,0,0))+'^1'
  add('script','true',frame,frame)
 reset();props();add('fwalk',fbits(-90),127);add('fclear');route([]);add('ffollow',0);props();add('fwalk',fbits(0),0)
 # Plain, diagonal, shifted, edge, rising and descending paths; advance and skip
 # callbacks, remaining vectors, correction/clear/follow retention and stalls.
 for scene in ['flat','turn','shift','edge','rise','drop','wall','stalled']:
  reset()
  for i in range(10):
   y=i if scene=='rise'else-i if scene=='drop'else 0;ty=y+1 if scene=='rise'else y-1 if scene=='drop'else y;z=max(0,i-4)if scene=='turn'else 0
   step(i,(i,y,z),(i+1,ty,z+1 if scene=='turn'else z),'ASCEND'if scene=='rise'else'FALL'if scene=='drop'else'TRAVERSE',flags=1 if scene=='shift'else 2 if scene=='edge'else 0)
  route(range(10));add('ffollow',0)
  if scene=='wall':at(2,0,0,1);at(2,1,0,1)
  for tick in range(46):
   x=.5 if scene=='stalled'else.5+tick*.24;y=(int(x)if scene=='rise'else-int(x)if scene=='drop'else 0);z=max(0,int(x)-4)+.5 if scene=='turn'else.5
   player(tick,(x,y,z),(.15,0,0),flags=3 if tick%3==0 else 1,support=(int(x),y-1,int(z)))
   script(x,y,z,scene!='stalled');add('fwalk',fbits(-90),64 if tick%2 else 0)
   if tick%7==0:props()
   if tick==13:add('fcorrect')
  add('ffollow',2);props();add('fclear');props()
 # Real action orchestration and waiting on a server floor; timeouts use the
 # actual original BlockActions and body commands rather than a summary stub.
 for scene in ['break','place','pillar','floor','open','unconfirmed']:
  reset();places=[(1,-1,0)]if scene in ['floor','unconfirmed']else[(1,0,0)]if scene in ['place','pillar']else[]
  step(0,to=(1,1,0)if scene=='pillar'else(1,0,0),move='PILLAR'if scene=='pillar'else'TRAVERSE',breaks=[(1,0,0)]if scene=='break'else[],places=places,opens=[(1,0,0)]if scene=='open'else[]);step(1,(1,0,0),(2,0,0));route([0,1]);add('ffollow',0)
  at(1,0,0,11 if scene=='open'else 1 if scene=='break'else 0)
  if places:at(*places[0],0)
  if scene=='unconfirmed':at(1,-2,0,0)
  add('targetstack',0,'false','false',fbits(1),0,'true','false');add('targetsetup',0,bits(4.5),17,fbits(0),'1,-1,0'if scene!='break'and scene!='open'else'1,0,0','UP','true',2**31-1)
  for tick in range(66):
   player(tick,position=(.9,.8 if scene=='pillar'else 0,.5),delta=(.05,0,0));script(.9,.8 if scene=='pillar'else 0,.5)
   if tick==3:
    if scene=='break':at(1,0,0,0)
    if scene=='open':at(1,0,0,10)
    if scene=='unconfirmed':at(1,-1,0,1)
   if tick==14:add('aanswer','1,0,0'if scene in ['break','open','place','pillar']else'1,-1,0',1)
   add('atick');add('aaim');add('fwalk',fbits(-90),64)
   if tick%10==0:props()
  add('fcorrect');add('fclear')
 # Falls and bridge-rejection retry gates, external floors with a persistent
 # straight bridge line, all bridge methods and successful/failed simulations.
 methods=['AUTO','NINJA','GODBRIDGE','MOONWALK','BREEZILY','ONE_STACK','SWIFT_SNEAK','TELLY']
 for method,external,fall in itertools.product(methods,[False,True],[False,True]):
  reset()
  for i in range(8):step(i,(i,0,0),(i+1,0,0),places=[(i+1,-1,0)])
  step(8,(8,0,0),(9,0,1));route(range(9));add('ffollow',0);add('bridgenew',fbits(-90),fbits(75),method,'false',bits(.7 if method=='SWIFT_SNEAK'else.3));add('fexternal',b(external))
  for tick in range(14):
   x=.5+tick*.5;y=-2 if fall else 0;player(tick,(x,y,.5),(.1,0,0),flags=0 if fall and tick<3 else 1);script(x,y,.5,progress=tick%3!=0);add('fwalk',fbits(-90),64);add('fruns')
  add('fclear')
 # Swim/breath/climb/leap selection with the original player and body reads.
 for movement,flags,air in itertools.product(['SWIM','CLIMB_UP','CLIMB_DOWN','PARKOUR'],[0,1,16,144,32],[50,200,300]):
  reset();step(0,to=(3,1 if movement in ['CLIMB_UP','PARKOUR']else 0,0),move=movement,flags=4 if movement=='SWIM'else 8 if movement=='PARKOUR'else 0);route([0]);add('ffollow',0)
  if movement.startswith('CLIMB'):at(0,0,0,4);at(0,1,0,4)
  for tick in range(4):player(tick,position=(.5,0,.5),delta=(.1,0,0),flags=flags,air=air);script();add('fwalk',fbits(-90),0);add('foff')
 # Equal coordinates from a distinct route feet object must retain !==. This
 # far target allows Hops to start, making the identity decision observable.
 reset();step(0,to=(5,0,0));step(1,to=(5,0,0));route([0,1]);add('ffollow',0);player(1,flags=3,food=20);script();add('fwalk',fbits(-90),0);add('ffollow',0);add('fwalk',fbits(-90),0);add('fcorrect');add('fwalk',fbits(-90),0)
