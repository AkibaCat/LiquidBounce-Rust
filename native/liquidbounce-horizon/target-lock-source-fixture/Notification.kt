package net.ccbluex.liquidbounce.utils.client
import fixture.Fixture
import net.ccbluex.liquidbounce.event.events.NotificationEvent
fun notification(title:String,message:String,severity:NotificationEvent.Severity){
 Fixture.trace.add("notify:"+message+":"+Fixture.locks().size)
 Fixture.noticeTime?.let{Fixture.set("Temporary.MaximumTime",it)}
 Fixture.noticeClock?.let{Fixture.now=it}
}
