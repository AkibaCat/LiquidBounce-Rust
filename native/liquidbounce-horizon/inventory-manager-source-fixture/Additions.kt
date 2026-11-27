package net.ccbluex.liquidbounce.additions
import net.ccbluex.liquidbounce.utils.client.Mouse
fun Mouse.setPosition(x:Double,y:Double) { this.x=x;this.y=y;fixture.Harness.effects.add("cursor:${x.toRawBits()}:${y.toRawBits()}") }
