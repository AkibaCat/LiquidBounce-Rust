/* SPDX-License-Identifier: GPL-3.0-or-later */
import com.google.gson.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.regex.*;
public class JdkRegexLayout {
 static JsonArray behindNodes(Object root)throws Exception {
  JsonArray out=new JsonArray();Set<Object> seen=Collections.newSetFromMap(new IdentityHashMap<>());
  ArrayDeque<Object> pending=new ArrayDeque<>();pending.add(root);
  while(!pending.isEmpty()) {
   Object node=pending.removeFirst();if(!seen.add(node))continue;
   String kind=node.getClass().getSimpleName();
   if(kind.equals("Behind")||kind.equals("BehindS")||kind.equals("NotBehind")||kind.equals("NotBehindS")) {
    JsonObject row=new JsonObject();row.addProperty("kind",kind);
    for(String name:List.of("rmin","rmax")) {
     Class<?> owner=node.getClass();Field field=null;
     while(owner!=null)try{field=owner.getDeclaredField(name);break;}catch(NoSuchFieldException e){owner=owner.getSuperclass();}
     field.setAccessible(true);row.addProperty(name,field.getInt(node));
    }
    out.add(row);
   }
   for(Class<?> owner=node.getClass();owner!=null&&owner.getName().startsWith("java.util.regex.Pattern$");owner=owner.getSuperclass())
    for(Field field:owner.getDeclaredFields()) {
     if(Modifier.isStatic(field.getModifiers()))continue;
     String name=field.getType().getName();
     if(name.startsWith("java.util.regex.Pattern$")||name.startsWith("[Ljava.util.regex.Pattern$")) {
      field.setAccessible(true);Object next=field.get(node);
      if(next instanceof Object[] nodes){for(Object value:nodes)if(value!=null)pending.add(value);}
      else if(next!=null)pending.add(next);
     }
    }
  }return out;
 }
 public static void main(String[] args)throws Exception {
  TreeSet<String> patterns=new TreeSet<>();
  for(var row:JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonArray())
   for(var p:row.getAsJsonObject().getAsJsonArray("patterns"))if(!p.isJsonNull())patterns.add(p.getAsString());
  Field root=Pattern.class.getDeclaredField("root"),supplementary=Pattern.class.getDeclaredField("hasSupplementary");
  root.setAccessible(true);supplementary.setAccessible(true);
  JsonArray out=new JsonArray();
  for(String text:patterns)try {
   Pattern pattern=Pattern.compile(text);JsonObject row=new JsonObject();row.addProperty("pattern",text);
   row.addProperty("root",root.get(pattern).getClass().getSimpleName());row.addProperty("hasSupplementary",supplementary.getBoolean(pattern));out.add(row);
   row.add("lookbehinds",behindNodes(root.get(pattern)));
  }catch(PatternSyntaxException e){}
  System.out.write(new Gson().toJson(out).getBytes(java.nio.charset.StandardCharsets.UTF_8));
 }
}
