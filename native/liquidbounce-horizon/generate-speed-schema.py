#!/usr/bin/env python3
"""Pinned ModuleSpeed's three independent choices and complete option tree."""
from pathlib import Path
import copy,json
HERE=Path(__file__).resolve().parent
def leaf(name,kind,default,lo=None,hi=None,options=None):
 n=dict(name=name,kind=kind,default=default)
 if lo is not None:n['range']={'from':lo,'to':hi}
 if options is not None:n['options']=options
 return n
def b(n,d=True):return leaf(n,'BOOLEAN',d)
def f(n,d,lo,hi):return leaf(n,'FLOAT',d,lo,hi)
def i(n,d,lo,hi):return leaf(n,'INT',d,lo,hi)
def g(n,*c):return dict(name=n,kind='CONFIGURABLE',children=list(c))
def toggle(n,d,*c):return dict(name=n,kind='TOGGLEABLE',children=[b('Enabled',d),*c])
def choice(n,d,*m):return dict(name=n,kind='CHOICE',default=d,modes=list(m))
MODES=[
 g('LegitHop'),
 g('Custom',toggle('HorizontalModification',True,f('HorizontalAcceleration',0,-.1,.2),f('HorizontalJumpOff',0,-.5,1),i('TicksToBoostOff',0,0,20)),toggle('VerticalModification',True,f('JumpHeight',.42,0,3),f('Pulldown',0,0,1),f('PullDownDuringFall',0,0,1)),f('TimerSpeed',1,.1,10),toggle('Strafe',True,f('Strength',1,.1,1),b('CustomSpeed',False),f('Speed',1,.1,10),i('VelocityTimeout',0,0,20),b('StrafeKnock',False))),
 g('YPort',f('Speed',.4,.1,1)),
 g('PiercingAttack',leaf('HoldTime','INT_RANGE',[1,1],1,20),b('OnGround'),b('IgnoreHunger',False),b('WaitForCooldown'),i('MinimumDurability',1,0,20)),
 g('Verus-b3882'),g('HypixelBHop',b('HorizontalAcceleration'),b('VerticalAcceleration')),g('HypixelLowHop',b('Glide',False)),
 g('Spartan-4.0.4.3'),g('Spartan-4.0.4.3-FastFall'),g('CubeCraft-2024-06-30',f('Speed',.5,.1,5),i('ReboostTicks',30,10,50)),
 g('Vulcan-2.8.6'),g('Vulcan-2.8.8'),g('Vulcan-2.8.6-Ground'),g('Grim-2.3.73',f('BoostSpeed',.08,.01,.08),f('ShrinkBox',.5,.1,2)),
 g('NCP-2024-08-31',toggle('PullDown',True,f('MotionMultiplier',1,.01,10),i('OnTick',5,1,9),b('OnHurt')),toggle('Boost',True,f('InitialBoostMultiplier',1,.01,10)),b('Timer'),b('DamageBoost'),b('LowHop'),b('AirStrafe')),
 g('Intave-14',toggle('Strafe',True,f('Strength',.27,.01,.27)),toggle('AirBoost',True)),g('Intave-14-Fast',b('Timer')),
 g('HylexLowHop'),g('HylexGround'),g('BlocksMC',b('RoundStrafeYaw',False)),g('Matrix-7'),g('Rewound',b('Timer')),
]
NOT=['WhileUsingItem','DuringScaffold','WhileSneaking','IsFallFlying','IsInLiquid']
SCHEMA=[choice('Mode','LegitHop',*MODES),leaf('Not','MULTI_CHOOSE',['DuringScaffold'],options=NOT),b('AvoidEdgeBump'),toggle('OnlyInCombat',False,choice('Mode','LegitHop',*copy.deepcopy(MODES))),toggle('OnlyOnPotionEffect',False,choice('PotionEffect','Speed',g('Speed',leaf('LevelRange','INT_RANGE',[1,2],0,4)),g('Slowness',leaf('LevelRange','INT_RANGE',[1,2],0,4)),g('Both',leaf('PotionEffectsBoostRange','FLOAT_RANGE',[1.15,1.35],.25,2))),choice('Mode','LegitHop',*copy.deepcopy(MODES))),toggle('YawOffset',False,leaf('YawOffsetMode','CHOOSE','Air',options=['Ground','Air','Constant']),dict(name='Rotations',kind='ROTATIONS',children=[]))]
SCHEMA[1]['canBeNone']=True
if __name__=='__main__':
 p=HERE.parent/'liquidbounce-behaviors/src/speed_schema.json'
 p.write_text(json.dumps(SCHEMA,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 print(f'{p}: 22 source choices in each of three independent scopes')
