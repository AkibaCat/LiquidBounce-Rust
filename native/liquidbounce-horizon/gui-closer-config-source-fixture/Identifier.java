package net.minecraft.resources;
public class Identifier {
    private final String name;private Identifier(String name){this.name=name;}
    public static Identifier parse(String name){return new Identifier(name);}
    public String toString(){return name;}
}
