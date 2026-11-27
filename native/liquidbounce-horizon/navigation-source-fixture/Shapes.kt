package fixture
import java.io.File
import net.ccbluex.agent.math.*
import net.ccbluex.agent.world.block.*
private fun d(raw:String)=Double.fromBits(raw.toULong().toLong())
private fun v(raw:String)=raw.split(',').let{Vec3(d(it[0]),d(it[1]),d(it[2]))}
private fun b(raw:String)=raw.split(',').let{AABB(d(it[0]),d(it[1]),d(it[2]),d(it[3]),d(it[4]),d(it[5]))}
private fun s(raw:String)=VoxelShape(if(raw=="-")DoubleArray(0)else raw.split(',').map(::d).toDoubleArray())
private fun Vec3.raw()=listOf(x,y,z).joinToString(","){it.toRawBits().toULong().toString()}
private fun AABB.raw()=listOf(minX,minY,minZ,maxX,maxY,maxZ).joinToString(","){it.toRawBits().toULong().toString()}
fun main(){
 val results=File(System.getProperty("commands")).readLines().map{command->val p=command.split(' ');when(p[0]){
  "box"->{val a=b(p[1]);val other=b(p[2]);val point=v(p[3]);val offset=v(p[4]);val clip=a.clip(point,offset);listOf(a.raw(),"${a.xsize.toRawBits().toULong()},${a.ysize.toRawBits().toULong()},${a.zsize.toRawBits().toULong()}",a.center.raw(),a.bottomCenter.raw(),a.move(offset).raw(),a.inflate(offset.x,offset.y,offset.z).raw(),a.expandTowards(offset).raw(),a.minmax(other).raw(),a.intersect(other).raw(),a.intersects(other),a.contains(point),a.distanceToSqr(point).toRawBits().toULong(),clip?.raw()?:"-",a.setMinY(offset.y).raw(),a.setMaxY(offset.y).raw(),AABB.ofSize(point,offset.x,offset.y,offset.z).raw(),AABB.unitCubeFromLowerCorner(point).raw()).joinToString("|")}
  "shape"->{val shape=s(p[1]);val axis=Direction.Axis.entries[p[2].toInt()];val body=b(p[3]);val distance=d(p[4]);val offset=v(p[5]);val moved=shape.move(offset.x,offset.y,offset.z);listOf(shape.isEmpty,shape.boxCount,shape.toArray().joinToString(","){it.toRawBits().toULong().toString()},shape.toAabbs().joinToString(";"){it.raw()},Direction.Axis.entries.joinToString(","){shape.min(it).toRawBits().toULong().toString()},Direction.Axis.entries.joinToString(","){shape.max(it).toRawBits().toULong().toString()},if(shape.isEmpty)"-"else shape.bounds().raw(),shape.collide(axis,body,distance).toRawBits().toULong(),moved.toArray().joinToString(","){it.toRawBits().toULong().toString()},shape===moved,shape==s(p[6]),Shapes.collide(axis,body,listOf(shape,moved),distance).toRawBits().toULong()).joinToString("|")}
  "ray"->{val clip=Clip(v(p[1]),v(p[2]));if(p[3]!="-")p[3].split(';').forEach{clip.box(b(it))};"${clip.face?.name?:"-"}|${clip.location().raw()}"}
  "identity"->{val coords=p[1].split(',').map(::d);val shape=Shapes.box(coords[0],coords[1],coords[2],coords[3],coords[4],coords[5]);"${shape===Shapes.block()}|${Shapes.empty()===Shapes.empty().move(1.0,2.0,3.0)}"}
  else->error(command)
 }}
 File(System.getProperty("results")).writeText(results.joinToString("\n")+"\n")
}
