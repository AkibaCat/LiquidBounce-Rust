"""Three small usability scenes; full vector validation is deferred."""
def generate(add,new,player,b,bits,raw,fbits):
 for mask,method in [(0,'AUTO'),(7,'NINJA'),(63,'TELLY')]:
  new();add('targetnew');add('bridgenew',0,0,method,'false',bits(.3))
  add('capopts',mask,3,bits(5.5),fbits(18.),'true',method,'true')
  add('capsets','1,-2','8,12','1024,-2048')
  for slot in [*range(9),40]:
   add('targetstack',slot,'false',b(slot==0),fbits(8. if slot==0 else 1.),2 if slot==0 else 0,b(slot in [1,40]),'false')
   add('capstack',slot,16 if slot==40 else 4,b(slot not in [0,1,40]))
  player(food=20,air=250)
  add('capsafe');add('caphot');add('capwalk')
  add('capsets','99','88','77');add('capquery')
  for hardness in [-1.,0.,1.]:
   add('caphard',1,fbits(hardness),'true');add('capticks',1);add('capbreak',1)
  add('targetstack',0,'false','true',fbits(100.),32,'true','false');add('capstack',0,64,'false')
  add('caphard',1,fbits(1.),'true');add('capticks',1)
  add('capopts',mask,3,bits(5.5),fbits(18.),'true',method,'false');add('capticks',1)
