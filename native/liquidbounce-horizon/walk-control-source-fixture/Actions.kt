// fixture code
private var actionLatency=0
private var actionUse=true
private var actionStart=true
private var actionContinue=true
private var actionBox=AABB(Vec3(4.0,0.0,4.0),Vec3(4.6,1.8,4.6))
private val retainedPlacements=ArrayList<PlacedBlock>()
private fun placementId(block:PlacedBlock?):String {if(block==null)return "-";val found=retainedPlacements.indexOfFirst{it===block};if(found>=0)return found.toString();retainedPlacements+=block;return retainedPlacements.lastIndex.toString()}
private fun actionEvent(kind:String,block:PlacedBlock){events+="$kind:${placementId(block)}:${block.pos.raw()}:${block.tick}:${block.verdict.tag}"}
private fun actionWarning(args:Array<out Any?>?){val text=args!![0]as String;val pos=args[1]as BlockPos;events+="warn:${if(text.startsWith("The server put"))"broken"else"placed"}:${pos.raw()}"}
private fun actionRaw():String {val action=field(actions,"action")?:return "-";val pos=(field(action,"pos")as BlockPos).raw();return when(action.javaClass.simpleName){"Mine"->"M:$pos";"Open"->"O:$pos";else->"P:$pos:${field(action,"ready")}:${(field(action,"held")as ViewRotation?)?.raw()?:"-"}:${(field(action,"plan")as BlockTargets.Plan?)?.raw()?:"-"}"}}
private fun actionSnapshot()=listOf(actionRaw(),actions.rotation?.raw()?:"-",field(actions,"ticks"),field(actions,"useCooldown"),(field(actions,"digging")as BlockPos?)?.raw()?:"-",actions.rejectedInRow,actions.rejections,placementId(actions.placement),retainedPlacements.mapIndexed{i,p->"$i:${p.pos.raw()}:${p.tick}:${p.verdict.tag}"}.joinToString(";")).joinToString("~")
private fun actionsExtra(a:List<String>):String {
 val result=when(a[0]){
 "anew"->{actions=BlockActions(ctx);actionLatency=0;actionUse=true;actionStart=true;actionContinue=true;actionBox=AABB(Vec3(4.0,0.0,4.0),Vec3(4.6,1.8,4.6));retainedPlacements.clear();"-"}
 "aset"->{actionLatency=a[1].toInt();actionUse=a[2].toBoolean();actionStart=a[3].toBoolean();actionContinue=a[4].toBoolean();actionBox=AABB(v(a[5]),v(a[6]));"-"}
 "amine"->{actions.mine(p(a[1]));"-"}
 "aopen"->{actions.open(p(a[1]));"-"}
 "aplace"->{actions.place(p(a[1]),a[2].toBoolean(),if(a[3]=="-")null else ViewRotation(f(a[3]),f(a[4])));"-"}
 "aidle"->{actions.idle();"-"}
 "aaim"->{actions.aim();"-"}
 "atick"->{actions.tick();"-"}
 "aanswer"->actions.answer(p(a[1]),states.getValue(a[2].toInt())).toString()
 "alevel"->{actions.levelChanged();"-"}
 "aforget"->{actions.forgetRejections();"-"}
 "astatus"->{val pos=p(a[1]);"${actions.isActive},${actions.isUnconfirmed(pos)},${actions.isOpenUnconfirmed(pos)},${pos.asLong()in actions.unbreakable}"}
 "aorder"->{@Suppress("UNCHECKED_CAST")val map=field(actions,"unconfirmed")as Map<BlockPos,PlacedBlock>;check(map.keys.toList()==ps(a[1]));"-"}
 else->error(a.joinToString(" "))
 }
 return result+"@"+actionSnapshot()
}
