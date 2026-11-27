package net.ccbluex.liquidbounce.utils.kotlin
import java.util.concurrent.ThreadLocalRandom
import kotlin.math.nextUp
import net.ccbluex.liquidbounce.event.Harness
enum class Priority { NOT_IMPORTANT, IMPORTANT_FOR_USAGE_1 }
// Same pinned ArrayExtensions.random body, with a draw counter around its
// real JDK25 call. ModuleDerp.kt itself is compiled byte-for-byte unchanged.
fun ClosedFloatingPointRange<Float>.random():Float {
    return if(start>=endInclusive)start else {
        Harness.draws++
        ThreadLocalRandom.current().nextFloat(start,endInclusive.nextUp())
    }
}
