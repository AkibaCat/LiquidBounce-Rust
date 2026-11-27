package fixture;
import com.google.gson.Gson;
import com.mojang.blaze3d.platform.InputConstants;
import java.lang.reflect.Field;
import java.util.*;
import net.minecraft.client.KeyMapping;

public final class KeyMappingClickOracle {
    static final LinkedHashMap<String,AccessibleKeyMapping> mappings=new LinkedHashMap<>();
    static final List<Map<String,Object>> rows=new ArrayList<>();
    static Field count;
    static void capture(String operation,String mapping,String key,Boolean held,Boolean consumed) throws Exception {
        Map<String,Object> row=new LinkedHashMap<>();row.put("operation",operation);row.put("mapping",mapping);row.put("key",key);
        if(held!=null)row.put("held",held);if(consumed!=null)row.put("consumed",consumed);
        Map<String,Object> counts=new LinkedHashMap<>(),down=new LinkedHashMap<>();
        for(var entry:mappings.entrySet()){counts.put(entry.getKey(),count.getInt(entry.getValue()));down.put(entry.getKey(),entry.getValue().isDown());}
        row.put("counts",counts);row.put("down",down);rows.add(row);
    }
    static void click(String key) throws Exception {
        var input=InputConstants.getKey(key);
        // Exact exclusion from the pinned LiquidBounce MixinKeyMapping (vanilla lacks this fix).
        if(!input.equals(InputConstants.UNKNOWN))KeyMapping.click(input);
        capture("click","",key,null,null);
    }
    static void bind(String name,String key) throws Exception {
        mappings.get(name).setKey(InputConstants.getKey(key));KeyMapping.resetMapping();capture("bind",name,key,null,null);
    }
    static void consume(String name) throws Exception {capture("consume",name,"",null,mappings.get(name).consumeClick());}
    static void held(String name,boolean value) throws Exception {mappings.get(name).setDown(value);capture("held",name,"",value,null);}
    static void release() throws Exception {KeyMapping.releaseAll();capture("release","","",null,null);}
    public static void main(String[] args) throws Exception {
        count=KeyMapping.class.getDeclaredField("clickCount");count.setAccessible(true);
        // MC 26.3 uses SDL mouse codes: left is 1.
        for(String name:List.of("key.attack","key.use","key.forward","key.screenshot","key.inventory","key.hotbar.1","key.sneak"))mappings.put(name,new AccessibleKeyMapping(name,1));
        release();held("key.attack",true);held("key.sneak",true);click("key.mouse.left");click("key.mouse.left");
        bind("key.attack","key.mouse.right");bind("key.use","key.keyboard.z");click("key.mouse.right");
        consume("key.attack");consume("key.use");consume("key.attack");consume("key.attack");consume("key.attack");
        bind("key.attack","key.keyboard.unknown");bind("key.use","key.keyboard.unknown");click("key.keyboard.unknown");
        consume("key.use");consume("key.forward");release();
        for(String name:mappings.keySet())bind(name,"key.keyboard.z");
        click("key.keyboard.z");held("key.attack",false);held("key.sneak",false);click("key.keyboard.z");
        for(String name:mappings.keySet()){consume(name);consume(name);consume(name);}
        release();System.out.println(new Gson().toJson(rows));
    }
}
