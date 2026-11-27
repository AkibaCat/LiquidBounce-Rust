# Pinned built-in command inventory

LiquidBounce ae0eb396, `CommandManager.registerInbuilt()`. This is an audit of the source tree, not a claim that all commands have been ported. Dynamic addon commands and Brigadier completion remain not connected.

| Root and aliases | Execution route | Pinned source under features/command |
| --- | --- | --- |
| `ping` | not_connected | `commands/ingame/CommandPing.kt` |
| `tps` | not_connected | `commands/ingame/CommandTps.kt` |
| `username` | not_connected | `commands/ingame/CommandUsername.kt` |
| `clear` | not_connected | `commands/client/CommandClear.kt` |
| `coordinates` (position, coords) | not_connected | `commands/ingame/CommandCoordinates.kt` |
| `map_image` | not_connected | `commands/ingame/CommandMapImage.kt` |
| `hide` | connected | `commands/client/CommandHide.kt` |
| `panic` | not_connected | `commands/client/CommandPanic.kt` |
| `say` | connected | `commands/ingame/CommandSay.kt` |
| `translate` (tr) | not_connected | `commands/translate/CommandTranslate.kt` |
| `autotranslate` | not_connected | `commands/translate/CommandAutoTranslate.kt` |
| `rename` | not_connected | `commands/ingame/creative/CommandItemRename.kt` |
| `marketplace` | not_connected | `commands/client/marketplace/CommandMarketplace.kt` |
| `toggle` (t) | connected | `commands/client/CommandToggle.kt` |
| `targets` (target, enemies, enemy) | not_connected | `commands/client/CommandTargets.kt` |
| `binds` | connected | `commands/client/CommandBinds.kt` |
| `autodisable` | connected | `commands/module/CommandAutoDisable.kt` |
| `invsee` | not_connected | `commands/module/CommandInvsee.kt` |
| `xray` | not_connected | `commands/module/CommandXRay.kt` |
| `value` | connected | `commands/client/CommandValue.kt` |
| `bind` | connected | `commands/client/CommandBind.kt` |
| `autoaccount` | connected | `commands/module/CommandAutoAccount.kt` |
| `center` | not_connected | `commands/ingame/CommandCenter.kt` |
| `goto` | not_connected | `commands/ingame/CommandGoto.kt` |
| `follow` | not_connected | `commands/ingame/CommandFollow.kt` |
| `help` | connected | `commands/client/CommandHelp.kt` |
| `remoteview` (rv) | not_connected | `commands/ingame/CommandRemoteView.kt` |
| `debug` | not_connected | `commands/client/CommandDebug.kt` |
| `friend` | connected | `commands/client/CommandFriend.kt` |
| `liquidchat` (lc) | not_connected | `commands/client/liquidchat/CommandLiquidChat.kt` |
| `client` | not_connected | `commands/client/client/CommandClient.kt` |
| `config` | not_connected | `commands/client/CommandConfig.kt` |
| `localconfig` | not_connected | `commands/client/CommandLocalConfig.kt` |
| `addon` | not_connected | `commands/client/CommandAddon.kt` |
| `fakeplayer` | not_connected | `commands/ingame/fakeplayer/CommandFakePlayer.kt` |
| `give` | not_connected | `commands/ingame/creative/CommandItemGive.kt` |
| `skull` | not_connected | `commands/ingame/creative/CommandItemSkull.kt` |
| `stack` | not_connected | `commands/ingame/creative/CommandItemStack.kt` |
| `enchant` | not_connected | `commands/ingame/creative/CommandItemEnchant.kt` |
| `vclip` | not_connected | `commands/ingame/teleport/CommandVClip.kt` |
| `teleport` (tp) | not_connected | `commands/ingame/teleport/CommandTeleport.kt` |
| `playerteleport` (playertp, ptp) | not_connected | `commands/ingame/teleport/CommandPlayerTeleport.kt` |
| `serverinfo` | not_connected | `commands/ingame/CommandServerInfo.kt` |
| `models` | not_connected | `commands/deeplearn/CommandModels.kt` |
