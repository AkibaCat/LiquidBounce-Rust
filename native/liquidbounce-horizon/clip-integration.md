# Clip

Source: ModuleClip.kt at ae0eb396, actual Movement category and Choice name retained.

Fancy searches loaded blocks, requires a wall before the first valid opening, uses the
canonical cached/dynamic support registry with SupportType::Center, and reads real
shulker and moving piston states through SceneCollisions. Air and Water/Lava facts
come from the actual resource tables. Forward/back/left/right precedence, sneak
before jump, yaw quadrants, five-tick CoroutineTicker waits and ▲▼ hotbar overlay
match the source. Both target and old-mode snaps update previous_position.

Old executes through the enable effect, supports negative offsets and optional
velocity reset, then disables synchronously. Changing to Old while enabled invokes
the same effect. Notifications and settings have simplified/traditional Chinese.

Fly guard queries the real registry if Fly is implemented; Fly is not registered as
a placeholder here. Native compile and game validation remain pending until the
batch report records them. Focused smoke covers wall search, input and cancellation.
