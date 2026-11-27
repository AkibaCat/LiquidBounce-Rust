package net.ccbluex.liquidbounce.utils.client
import fixture.Fixture
import net.ccbluex.liquidbounce.event.events.NotificationEvent
import org.slf4j.LoggerFactory
class Client {
    val player get()=if(Fixture.inGame)Fixture.player else null
    val isSameThread get()=Fixture.sameThread
    fun execute(task:Runnable){task.run()}
}
val mc=Client()
object NullableBypass{fun mc():Client?=mc}
val logger=LoggerFactory.getLogger("ResetVL oracle")
fun notification(title:String,message:String,severity:NotificationEvent.Severity){Fixture.notification(title,message,severity.name)}
