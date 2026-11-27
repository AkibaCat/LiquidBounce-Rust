// Actual Gson, EnumSet and CASE_INSENSITIVE_ORDER used by MultiChoiceListValue.
// The loop below follows the pinned Kotlin deserializeFrom statement order.
import com.google.gson.*;
import java.nio.file.*;
import java.util.*;

public final class MultiActionsFacts {
    enum Action {
        PLACE("PlacingWhileBreaking"), ATTACK("AttackingWhileUsing"), BREAK("BreakingWhileUsing");
        final String tag;
        Action(String tag) { this.tag = tag; }
    }
    public static void main(String[] args) throws Exception {
        var choices = new TreeMap<String, Action>(String.CASE_INSENSITIVE_ORDER);
        for (var action : Action.values()) choices.put(action.tag, action);
        var rows = new JsonArray();
        String[] inputs = {
            "null", "{}", "[]", "true", "17", "\"unknown\"",
            "\"placingwhilebreaking\"", "\"AttackıngWhileUsing\"", "\"PlacingWhileBreaKing\"",
            "[\"BreakingWhileUsing\",\"PlacingWhileBreaking\",\"BREAKINGWHILEUSING\"]",
            "[\"BreakingWhileUsing\",null]", "[\"AttackingWhileUsing\",{}]",
            "[[\"placingwhilebreaking\"],true,17]", "[[]]", "[[\"unknown\",\"unknown\"]]",
            "[\"AttackingWhileUsing\",[null]]", "[\"AttackingWhileUsing\",[[\"BreakingWhileUsing\"]]]"
        };
        for (var text : inputs) {
            var element = JsonParser.parseString(text);
            var active = EnumSet.allOf(Action.class);
            String error = null;
            try {
                active.clear();
                if (element instanceof JsonArray array) {
                    for (var entry : array) {
                        var choice = choices.get(entry.getAsString());
                        if (choice != null) active.add(choice);
                    }
                } else if (element instanceof JsonPrimitive primitive) {
                    var choice = choices.get(primitive.getAsString());
                    if (choice != null) active.add(choice);
                }
            } catch (RuntimeException failure) { error = failure.getClass().getSimpleName(); }
            int mask = 0;
            for (var action : active) mask |= 1 << action.ordinal();
            var row = new JsonObject(); row.add("input", element); row.addProperty("mask", mask);
            row.addProperty("error", error); rows.add(row);
        }
        var result = new JsonObject(); result.add("rows", rows);
        result.addProperty("defaultMask", 7); result.addProperty("canBeNone", true);
        result.addProperty("isOrderSensitive", false);
        Files.writeString(Path.of(args[0]), new GsonBuilder().setPrettyPrinting().serializeNulls().create().toJson(result));
    }
}
