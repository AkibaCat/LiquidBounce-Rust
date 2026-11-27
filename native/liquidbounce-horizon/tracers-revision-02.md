# Tracers revision 02

Root's integration review found the new ModuleKind tuple retained the copied `damageParticles` key. The original `ModuleTracers` uses the lower-camel module key `tracers`; the owned transactional integrator now registers `("Tracers", "Render", "tracers")`. The config/runtime/renderer implementation is unchanged and the focused smoke fixture already used the correct `liquidbounce.module.tracers` key. No Cargo or cumulative tests were rerun. The script's actual anchors are checked again. Preserve `tracers-freeze.json` as historical and use `tracers-freeze-02.json`.
