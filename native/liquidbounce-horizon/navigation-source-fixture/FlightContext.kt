package net.ccbluex.agent.navigation
class FlightOptions(val canFly:Boolean)
internal class NavigationContext(val body:Body,val options:()->FlightOptions)
