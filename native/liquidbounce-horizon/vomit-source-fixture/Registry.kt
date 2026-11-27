package net.minecraft.core.registries
import java.util.Optional
import net.minecraft.util.RandomSource
class Block(val index:Int)
class Holder(private val block:Block) { fun value()=block }
class Registry { var size=100;fun getRandom(random:RandomSource):Optional<Holder> = Optional.of(Holder(Block(random.nextInt(size)))) }
object BuiltInRegistries { val BLOCK=Registry() }
