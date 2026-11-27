package net.ccbluex.fastutil
import net.ccbluex.liquidbounce.config.types.ValueType
fun enumMapOf(vararg entries:Any):Map<ValueType,String> = entries.toList().chunked(2).associate { it[0] as ValueType to it[1] as String }
