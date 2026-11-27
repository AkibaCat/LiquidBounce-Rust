/* SPDX-License-Identifier: GPL-3.0-or-later
 * Generate observable JDK25 character facts. No JDK source is copied.
 */
import java.io.*;
import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;

public final class JdkRegexFacts {
    static final int LIMIT=0x110000;
    static final Class<?> CP;
    static final Method PREDICATE;
    static {
        try {
            CP=Class.forName("java.util.regex.CharPredicates");
            PREDICATE=Class.forName("java.util.regex.Pattern$CharPredicate").getDeclaredMethod("is",int.class);
            PREDICATE.setAccessible(true);
        } catch (Exception e) { throw new ExceptionInInitializerError(e); }
    }
    static Object call(String method, String name, boolean ci) throws Exception {
        Method m=CP.getDeclaredMethod(method,String.class,boolean.class);m.setAccessible(true);
        return m.invoke(null,name,ci);
    }
    static void ranges(DataOutputStream out, java.util.function.IntToLongFunction query) throws IOException {
        long previous=-1;
        for(int cp=0;cp<LIMIT;cp++) {
            long value=query.applyAsLong(cp);
            if(value!=previous){out.writeInt(cp);out.writeLong(value);previous=value;}
        }
    }
    static String rust(String text) {return "\""+text.replace("\\","\\\\").replace("\"","\\\"")+"\"";}
    @SuppressWarnings("unchecked")
    static Map<String,Object> aliases(Class<?> type, String fieldName) throws Exception {
        Field f=type.getDeclaredField(fieldName);f.setAccessible(true);return (Map<String,Object>)f.get(null);
    }
    static void names(Path out) throws Exception {
        TreeMap<String,Integer> names=new TreeMap<>();
        for(int cp=0;cp<LIMIT;cp++) {String name=Character.getName(cp);if(name!=null)names.put(name,cp);}
        ByteArrayOutputStream text=new ByteArrayOutputStream();
        try(DataOutputStream index=new DataOutputStream(Files.newOutputStream(out.resolve("names-index.bin")))) {
            for(var e:names.entrySet()) {
                index.writeInt(text.size());index.writeInt(e.getValue());
                text.write(e.getKey().getBytes(StandardCharsets.UTF_8));text.write(0);
            }
        }
        Files.write(out.resolve("names.bin"),text.toByteArray());
    }
    public static void main(String[] args) throws Exception {
        Path out=Path.of(args[0]);Files.createDirectories(out);
        TreeMap<String,Object> predicates=new TreeMap<>();
        String direct="L M N Z C P S Lu Ll Lt Lm Lo Mn Me Mc Nd Nl No Zs Zl Zp Cc Cf Co Cs Cn Pc Pd Ps Pe Pi Pf Po Sm Sc Sk So LC LD L1 all ASCII Alnum Alpha Blank Cntrl Digit Graph Lower Print Punct Space Upper XDigit javaLowerCase javaUpperCase javaTitleCase javaDigit javaDefined javaLetter javaLetterOrDigit javaJavaIdentifierStart javaJavaIdentifierPart javaUnicodeIdentifierStart javaUnicodeIdentifierPart javaIdentifierIgnorable javaSpaceChar javaWhitespace javaISOControl javaMirrored javaAlphabetic javaIdeographic";
        TreeSet<String> unicode=new TreeSet<>();
        for(Method m:CP.getDeclaredMethods())if(m.getParameterCount()==0&&m.getName().matches("[A-Z_]+")) {
            unicode.add(m.getName());unicode.add(m.getName().replace("_",""));
        }
        unicode.addAll(Arrays.asList("ALPHA","LOWER","UPPER","TITLE","PUNCT","SPACE","XDIGIT","CNTRL","DIGIT"));
        for(boolean ci:new boolean[]{false,true}) {
            String suffix=ci?":1":":0";
            for(String name:direct.split(" ")) {
                Object p=call("forProperty",name,ci);if(p!=null)predicates.put("D:"+name+suffix,p);
            }
            for(String name:unicode) {
                Object p=call("forUnicodeProperty",name,ci);if(p!=null)predicates.put("U:"+name+suffix,p);
            }
            for(String name:"LOWER UPPER ASCII ALPHA DIGIT ALNUM PUNCT GRAPH PRINT BLANK CNTRL XDIGIT SPACE".split(" ")) {
                Object p=call("forPOSIXName",name,ci);if(p!=null)predicates.put("P:"+name+suffix,p);
            }
        }
        StringBuilder manifest=new StringBuilder("// Generated from the actual pinned JDK25 runtime; big-endian binary records.\n");
        manifest.append("pub static PROPERTIES: &[(&str,usize,usize)] = &[\n");
        int offset=0;
        try(DataOutputStream outData=new DataOutputStream(Files.newOutputStream(out.resolve("properties.bin")))) {
            for(var e:predicates.entrySet()) {
                int start=offset;boolean inside=false;
                for(int cp=0;cp<=LIMIT;cp++) {
                    boolean yes=cp<LIMIT&&(boolean)PREDICATE.invoke(e.getValue(),cp);
                    if(yes!=inside){outData.writeInt(cp);offset+=4;inside=yes;}
                }
                manifest.append("    ("+rust(e.getKey())+","+start+","+(offset-start)/8+"),\n");
            }
        }
        manifest.append("];\n");
        Class<?> bmp=Class.forName("java.util.regex.Pattern$BmpCharPredicate");
        manifest.append("pub static BMP_PROPERTIES: &[&str] = &[\n");
        for(var e:predicates.entrySet())if(bmp.isInstance(e.getValue()))manifest.append("    "+rust(e.getKey())+",\n");
        manifest.append("];\n");
        Map<Object,Integer> blocks=new IdentityHashMap<>();
        Map<String,Object> blockAliases=aliases(Character.UnicodeBlock.class,"map");
        List<Object> sortedBlocks=new ArrayList<>(new HashSet<>(blockAliases.values()));
        sortedBlocks.sort(Comparator.comparing(Object::toString));
        for(int i=0;i<sortedBlocks.size();i++)blocks.put(sortedBlocks.get(i),i+1);
        manifest.append("pub static BLOCKS: &[(&str,u64)] = &[\n");
        for(var e:new TreeMap<>(blockAliases).entrySet())manifest.append("    ("+rust(e.getKey())+","+blocks.get(e.getValue())+"),\n");
        manifest.append("];\npub static SCRIPTS: &[(&str,u64)] = &[\n");
        TreeMap<String,Object> scriptAliases=new TreeMap<>(aliases(Character.UnicodeScript.class,"aliases"));
        for(var s:Character.UnicodeScript.values())scriptAliases.put(s.name(),s);
        for(var e:scriptAliases.entrySet())manifest.append("    ("+rust(e.getKey())+","+((Enum<?>)e.getValue()).ordinal()+"),\n");
        manifest.append("];\n");
        Class<?> grapheme=Class.forName("jdk.internal.util.regex.Grapheme");
        Method type=grapheme.getDeclaredMethod("getType",int.class);type.setAccessible(true);
        Class<?> indic=Class.forName("jdk.internal.util.regex.IndicConjunctBreak");
        Method linker=indic.getDeclaredMethod("isLinker",int.class),extend=indic.getDeclaredMethod("isExtend",int.class),consonant=indic.getDeclaredMethod("isConsonant",int.class);
        linker.setAccessible(true);extend.setAccessible(true);consonant.setAccessible(true);
        Field rulesField=grapheme.getDeclaredField("rules");rulesField.setAccessible(true);
        boolean[][] rules=(boolean[][])rulesField.get(null);
        manifest.append("pub static GRAPHEME_RULES: [[bool;15];15] = [\n");
        for(boolean[] row:rules){manifest.append("[");for(boolean x:row)manifest.append(x+",");manifest.append("],\n");}
        manifest.append("];\n");
        try(DataOutputStream data=new DataOutputStream(Files.newOutputStream(out.resolve("characters.bin")))) {
            ranges(data,cp->{try {
                long value=(int)type.invoke(null,cp);
                if((boolean)linker.invoke(null,cp))value|=1L<<4;
                if((boolean)extend.invoke(null,cp))value|=1L<<5;
                if((boolean)consonant.invoke(null,cp))value|=1L<<6;
                if(Character.getType(cp)==Character.NON_SPACING_MARK)value|=1L<<7;
                if(Character.isAlphabetic(cp))value|=1L<<8;
                value|=(long)Character.getType(cp)<<9;
                value|=(long)Character.UnicodeScript.of(cp).ordinal()<<14;
                value|=(long)blocks.getOrDefault(Character.UnicodeBlock.of(cp),0)<<23;
                return value;
            }catch(Exception ex){throw new RuntimeException(ex);}});
        }
        try(DataOutputStream data=new DataOutputStream(Files.newOutputStream(out.resolve("case.bin")))) {
            for(int cp=0;cp<LIMIT;cp++) {
                int upper=Character.toUpperCase(cp),lower=Character.toLowerCase(cp);
                if(cp!=upper||cp!=lower){data.writeInt(cp);data.writeInt(upper);data.writeInt(lower);}
            }
        }
        names(out);
        Files.writeString(out.resolve("manifest.rs"),manifest,StandardCharsets.UTF_8);
        System.out.println("JDK "+System.getProperty("java.runtime.version")+"; predicates="+predicates.size());
    }
}
