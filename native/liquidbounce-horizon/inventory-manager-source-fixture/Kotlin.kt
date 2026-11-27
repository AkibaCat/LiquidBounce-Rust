package net.ccbluex.liquidbounce.utils.kotlin
import kotlinx.coroutines.*
val Dispatchers.Minecraft:MainCoroutineDispatcher get()=fixture.Clock
object EventPriorityConvention { const val READ_FINAL_STATE:Short=0 }
enum class Priority { LOW,NORMAL,HIGH,CRITICAL }
