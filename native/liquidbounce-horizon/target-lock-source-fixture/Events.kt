package net.ccbluex.liquidbounce.event.events
class AttackEntityEvent(val entity:Any,val cancelled:Boolean=false)
class GameTickEvent
class NotificationEvent{enum class Severity{INFO}}
class TagEntityEvent(val entity:Any){var decision="Unchanged";fun dontTarget(){decision="DontTarget"};fun ignore(){decision="Ignore"}}
