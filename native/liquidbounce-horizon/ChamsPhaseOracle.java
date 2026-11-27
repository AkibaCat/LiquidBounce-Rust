// SPDX-License-Identifier: GPL-3.0-or-later
// Exercises official 26.3 phase dispatch with a counting renderer and no GPU.
import java.lang.reflect.*;
import java.util.*;
import net.minecraft.client.renderer.SubmitNodeStorage;
import net.minecraft.client.renderer.feature.*;
import com.mojang.renderpearl.api.commands.RenderPass;

public final class ChamsPhaseOracle {
    private static Field field(Class<?> type, String name) throws Exception {
        Field field = type.getDeclaredField(name);
        field.setAccessible(true);
        return field;
    }
    private static Object allocate(Class<?> type) throws Exception {
        Class<?> unsafeType = Class.forName("sun.misc.Unsafe");
        Object unsafe = field(unsafeType, "theUnsafe").get(null);
        return unsafeType.getMethod("allocateInstance", Class.class).invoke(unsafe, type);
    }
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static int run(boolean improved, boolean solid) throws Exception {
        var storage = new SubmitNodeStorage();
        storage.setUseImprovedTransparency(improved);
        var collection = storage.order(0);
        Object phase = solid ? collection.solid : collection.translucentModels;
        var featureType = FeatureRendererType.create("chams-phase-oracle");
        int[] calls = {0};
        FeatureRenderer renderer = (FeatureRenderer) Proxy.newProxyInstance(
            ChamsPhaseOracle.class.getClassLoader(), new Class<?>[]{FeatureRenderer.class},
            (proxy, method, args) -> {
                if (method.getName().equals("executeGroup")) {
                    if (args[1] != null) throw new AssertionError("renderAllFeatures supplied an OIT stage");
                    calls[0]++;
                }
                return null;
            });
        var rendererMap = new FeatureRendererMap();
        rendererMap.put(featureType, renderer);
        var dispatcher = (FeatureRenderDispatcher) allocate(FeatureRenderDispatcher.class);
        field(FeatureRenderDispatcher.class, "featureRenderers").set(dispatcher, rendererMap);
        var frame = dispatcher.new PreparedFrame();
        field(frame.getClass(), "context").set(frame,
            new FeatureFrameContext(null, null, null, null, null, null, null, null));
        field(frame.getClass(), "submitNodeStorage").set(frame, storage);
        List submits = (List) field(frame.getClass(), "allSubmits").get(frame);
        submits.add(null);
        Class<?> groupType = Class.forName("net.minecraft.client.renderer.feature.FeatureRenderDispatcher$PreparedGroup");
        Constructor<?> groupConstructor = groupType.getDeclaredConstructor(
            int.class, FeatureRendererType.class, boolean.class, int.class, int.class);
        groupConstructor.setAccessible(true);
        Object group = groupConstructor.newInstance(0, featureType, false, 0, 0);
        Map groups = (Map) field(frame.getClass(), "groupsByPhase").get(frame);
        groups.put(phase, List.of(group));
        RenderPass pass = (RenderPass) Proxy.newProxyInstance(
            ChamsPhaseOracle.class.getClassLoader(), new Class<?>[]{RenderPass.class},
            (proxy, method, args) -> null);
        FeatureRenderDispatcher.renderAllFeatures(pass, frame);
        return calls[0];
    }
    public static void main(String[] args) throws Exception {
        int classicSolid = run(false, true);
        int classicTranslucent = run(false, false);
        int improvedSolid = run(true, true);
        int improvedTranslucent = run(true, false);
        if (classicSolid != 1 || classicTranslucent != 1 || improvedSolid != 1 || improvedTranslucent != 10)
            throw new AssertionError(Arrays.toString(new int[]{classicSolid, classicTranslucent, improvedSolid, improvedTranslucent}));
        System.out.println("{\"passed\":true,\"classicSolid\":" + classicSolid
            + ",\"classicTranslucent\":" + classicTranslucent
            + ",\"improvedSolid\":" + improvedSolid
            + ",\"improvedTranslucent\":" + improvedTranslucent
            + ",\"oitStage\":null}");
    }
}
