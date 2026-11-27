# ItemESP integration

Source: LiquidBounce `ae0eb396c07f22383789b28d5912d714c6ca0a86`,
`ModuleItemESP.kt`, `Legacy2DMarkerRenderer.kt`, `KeyedAabb.kt`,
`GenericStaticColorMode` / `GenericRainbowColorMode`, and `OutlineGlow`.

The owned behavior and configuration implement the original filter registry set,
eye-to-camera distance, collectible arrow subtypes, thrown tridents, tracers,
Glow / Box / Legacy2D, every glow style and its original defaults, fixed / rainbow
color, strict original-box connected-component merging, and all 16 legacy marker
rectangles. Native tick captures keep entity identity and survive removal until
the next tick; frame interpolation updates the same objects. Tracers and Glow
select from live objects every frame.

Native model extraction covers dropped items, every projectile in the creature
projectile collection, and standalone thrown tridents. The common OutlineGlow
registry resolves ESP / ItemESP / StorageESP outline ownership and color priority.
The model-specific hooks promote existing vanilla outlines before selecting the
owner and suppress duplicate ordinary outlines for captured entities. ItemESP
plain geometry uses its own renderer without depth testing.

Root serial integration order:

1. Publish and integrate the complete shared ESP / OutlineGlow changes.
2. `python -X utf8 native/liquidbounce-horizon/integrate-item-esp.py --check`
3. Run the same integrator without `--check`, once.
4. `python -X utf8 native/liquidbounce-horizon/update-item-esp-locales.py`
5. Keep the Step patch last in the complete generator.

Prepare only during development:

```powershell
python -X utf8 native/liquidbounce-horizon/test-item-esp.py --out build/item-esp-static-02
```

After a group of 5–8 modules is complete, run the official combined native / engine
check, then run the eight focused tests once:

```powershell
python -X utf8 native/liquidbounce-horizon/test-item-esp.py --out build/item-esp-focused-01 --run
```

Use `--filter <test-name>` to rerun a failing test alone. No focused test or live
game pass is claimed by this handoff. The fixture covers production model and
configuration; native geometry, shader interaction, language switching and final
visual parity remain subject to the combined build and final game validation.
