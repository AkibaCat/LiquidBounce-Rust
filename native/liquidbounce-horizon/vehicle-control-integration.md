# VehicleControl source port

Fixed source: LiquidBounce ae0eb396 ModuleVehicleControl (alias BoatFly).
All source settings are exposed: horizontal/vertical BaseSpeed and SprintSpeed,
Glide, MouseControl, NoGlideOnSprint, and Rehook with both timing settings.
Imported values preserve source bounds behavior rather than clamping silently.

The actual controlled boat or living mount is selected using local controller
and first-passenger rules. Movement uses physical sprint/jump/sneak keys and the
final directional event flags. MouseControl updates current and old vehicle yaw.
Boat water state comes from its actual push_by_fluid result. Active SpearKill
charge movement retains precedence. Airborne sneak is intercepted until a safe
dismount; the source one-shot rider detach remains a real input continuation.

Rehook uses the shared FIFO tick service, retained vehicle identity and the
source force-attempt sequence. It computes eye-to-box distance against the real
entity interaction attribute, sends ordinary main-hand interaction with world
border checks and conditional component-based swinging, and updates the real
rider link for force mount and detach. Missing and distant target messages and
the initial dismount hint are translated in both Chinese locales.

Single focused smoke: build/vehicle-control-smoke-01.exe,
complete_velocity_and_rehook_smoke passed. Official native compilation passed
in integrated-check-135-02.json. Its separate engine codec error was fixed in
the owned codec and awaits the next batch compilation. Full-client differential
and fresh game verification remain pending. No pristine upstream file is edited.
