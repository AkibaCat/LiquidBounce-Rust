package net.ccbluex.liquidbounce.utils.kotlin
class Priority(val priority:Int) { companion object {val IMPORTANT_FOR_USAGE_1=Priority(20);val NOT_IMPORTANT=Priority(-20)} }
object EventPriorityConvention { const val FIRST_PRIORITY:Short=1000;const val READ_FINAL_STATE:Short=-1000 }
