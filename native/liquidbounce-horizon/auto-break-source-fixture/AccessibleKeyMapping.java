package fixture;
import com.mojang.blaze3d.platform.InputConstants;
import net.minecraft.client.KeyMapping;
public final class AccessibleKeyMapping extends KeyMapping {
    public AccessibleKeyMapping(String name, int value) { super(name, InputConstants.Type.MOUSE, value, KeyMapping.Category.GAMEPLAY); }
    public InputConstants.Key getKey() { return key; }
}
