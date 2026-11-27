package fixture
import net.minecraft.client.gui.Font
import net.minecraft.world.item.ItemCooldowns
data class Entry(val endTick:Int,val currentTick:Int)
class Player {val cooldowns=ItemCooldowns()}
class Delta {fun getGameTimeDeltaPartialTick(pause:Boolean)=0f}
class MC {var player:Player?=Player();val deltaTracker=Delta();val font=Font()}
object Context {val mc=MC();var inGame=true;val ops=mutableListOf<Map<String,Any>>()}
