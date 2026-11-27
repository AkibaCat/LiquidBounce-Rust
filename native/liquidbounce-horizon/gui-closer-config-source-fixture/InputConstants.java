package com.mojang.blaze3d.platform;
public class InputConstants {
    public static Key getKey(String name){return new Key(name);}
    public static class Key {private final String name;public Key(String name){this.name=name;}public String getName(){return name;}}
}
