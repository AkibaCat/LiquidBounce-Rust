package fixture
import net.minecraft.client.KeyMapping
import net.minecraft.world.phys.HitResult
import net.ccbluex.liquidbounce.event.events.KeybindIsPressedEvent

class Options {
    // MC 26.3 uses SDL mouse codes: left is 1.
    val keyAttack=AccessibleKeyMapping("key.attack",1)
    val keyUse=AccessibleKeyMapping("key.use",1)
    val keyForward=AccessibleKeyMapping("key.forward",1)
    val keyScreenshot=AccessibleKeyMapping("key.screenshot",1)
}
class MC { val options=Options();var missTime=0;var hitResult:HitResult?=null }
class Interaction {var isDestroying=false}
class State(val isAir:Boolean)
object Context {
    val mc=MC();val interaction=Interaction();var state:State?=State(false)
    var handler:((KeybindIsPressedEvent)->Unit)?=null
}
