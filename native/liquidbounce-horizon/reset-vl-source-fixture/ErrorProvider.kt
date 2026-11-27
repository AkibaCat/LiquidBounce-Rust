package net.ccbluex.liquidbounce.utils.client.error
object ErrorHandler{fun fatal(error:Throwable,additionalMessage:String):Nothing=throw error}
