# LiquidBounce Rust Launcher

Rust + egui 原生启动器，固定为参考图的 300 × 450 逻辑像素、无边框布局。界面沿用 LiquidLauncher 的 Inter 字体、透明黑色 60% 背景、`#4677FF` 按钮及圆角控件。透明窗口不额外模拟系统背景模糊。中文使用 Windows 自带微软雅黑或黑体回退字体。

## 构建

在仓库根目录运行：

```powershell
cargo +stable build --release --manifest-path native/liquidbounce-launcher/Cargo.toml
```

生成文件：`native/liquidbounce-launcher/target/release/liquidbounce-launcher-egui.exe`。

把启动器放在发布包根目录，与 `runtime`、`theme`、`resources/26.3` 同级。启动器直接创建原生游戏进程，无需 PowerShell，默认进入游戏主菜单，由玩家选择单人世界或多人游戏。资源设置可切换到其他资源目录；启动前读取其中的 `horizon-resources.json`，确认 Minecraft 版本为 `26.3`，并检查游戏及 CEF 必需文件。

## 账户与运行

点击玩家名称可选择微软正版登录或离线账户。正版登录使用与 LiquidLauncher 相同的 `minecraft-auth 0.1.1`、微软应用配置及设备代码授权，用户在微软网页完成授权。Windows DPAPI 加密保存登录刷新状态；访问令牌不写入普通配置或命令行，仅经子进程标准输入传递。启动前会刷新即将过期的会话。登录恢复失败后需重新登录或明确选择离线账户。

设置保存于发布包内的 `user-data/launcher.json`，游戏输出位于 `user-data/launcher-game.log`。用户目录及游戏配置均保存在包内。游戏运行期间点击关闭或使用系统关闭命令会将启动器最小化，保留进程锁，防止相同用户目录被重复启动。

## 验证参数

```powershell
.\liquidbounce-launcher-egui.exe --launch-check .\launch-check.json
.\liquidbounce-launcher-egui.exe --smoke-report .\launcher-smoke.json
```

`--launch-check` 只验证发布包与资源，不创建窗口。`--smoke-report` 创建启动器窗口并在绘制数帧后退出；两个模式都不会启动游戏。可用 `--package PATH` 指定发布包根目录、`--resources PATH` 覆盖资源路径。
