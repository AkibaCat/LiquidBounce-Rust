import java.text.DecimalFormatSymbols;
import java.util.*;
import java.nio.charset.StandardCharsets;
// JDK25 DecimalFormatSymbols are the same symbols Formatter uses for %.1f.
class BetterInventoryLocaleFixture {
    public static void main(String[] args) throws Exception {
        var locales=new TreeMap<String,Locale>();
        for(var locale:Locale.getAvailableLocales())locales.put(locale.toLanguageTag().toLowerCase(Locale.ROOT),locale);
        StringBuilder out=new StringBuilder("{\"default\":\"").append(Locale.getDefault(Locale.Category.FORMAT).toLanguageTag()).append("\",\"rows\":[");
        boolean first=true;
        for(var entry:locales.entrySet()){
            var symbols=DecimalFormatSymbols.getInstance(entry.getValue());
            if(!first)out.append(',');first=false;
            out.append("{\"locale\":\"").append(entry.getKey()).append("\",\"decimal\":").append((int)symbols.getDecimalSeparator()).append(",\"zero\":").append((int)symbols.getZeroDigit()).append(",\"formats\":[");
            float[] values={0.5f,-0.5f,0.0f,-107374184.0f};
            for(int i=0;i<values.length;i++){if(i!=0)out.append(',');out.append('"').append(String.format(entry.getValue(),"%.1f",values[i])).append('"');}
            out.append("]}");
        }
        System.out.write(out.append("]}").toString().getBytes(StandardCharsets.UTF_8));
    }
}
