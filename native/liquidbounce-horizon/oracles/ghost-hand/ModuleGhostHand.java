// SPDX-License-Identifier: GPL-3.0-or-later
// Storage shell for compiling the untouched original TweakedMethods.java.
package net.ccbluex.liquidbounce.features.module.modules.exploit;
import java.util.Set;
import net.minecraft.world.level.block.Block;

public final class ModuleGhostHand {
    public static final ModuleGhostHand INSTANCE = new ModuleGhostHand();
    public boolean running;
    public Set<Block> targetedBlocks = Set.of();
    public boolean getRunning() { return running; }
    public Set<Block> getTargetedBlocks() { return targetedBlocks; }
}
