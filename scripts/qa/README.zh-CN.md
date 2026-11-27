<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# 原生启动和菜单烟测

使用 Python 3.11 或更高版本；Windows 故障堆栈诊断需要 x64 Python。所有测试都要求明确指定游戏程序、SHA-256、主题/CEF 包和资源目录，默认输出到项目的 `build/launcher-qa/`。每次使用新的 `--name`，已有结果不会被覆盖。

```powershell
python scripts/qa/test_game_lifecycle.py --binary <game.exe> --expected-sha256 <sha256> --package <package> --resources <resources/26.3> --name lifecycle-final --mode all
python scripts/qa/test_menu_api.py --binary <game.exe> --expected-sha256 <sha256> --package <package> --resources <resources/26.3> --name menu-api-final
```

包中需要 `theme/` 和 `runtime/`，后者包含 CEF 分发文件、locales 和同次构建的 `liquidbounce-cef-helper.exe`。游戏程序可以位于包外。可用 `--output-root` 指定其他测试输出目录。

`test_game_lifecycle.py` 验证菜单退出、进入世界后通过真实暂停菜单保存返回再退出、世界内直接退出，检查帧/逻辑推进及实际进程退出码。`test_menu_api.py` 验证主题自身的路由确认与后续新绘制帧、世界列表、服务器增删改排序及磁盘 NBT、离线会话与模块设置保存；该测试不会伪造页面确认。

测试仅创建自己的临时玩家、存档、配置和 CEF 缓存，使用回环 API 与离屏渲染，不操控桌面。服务器样本均为 `127.0.0.1`。出错时只诊断并清理该测试启动的进程。`capture_native_stacks.py --validate-only` 可只校验本机 Windows API 和结构布局；实际抓栈须明确传入 PID 和目标程序路径。

这些是启动流程与菜单 API 的基础验收，不代表全部模块的行为验收。详细结果写入各次测试的 `summary.json`、`result.json` 和控制记录。
