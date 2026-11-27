# 在本包内重新构建

此 source/ 是独立源码工作目录，包含当前移植的 native/ 源码与测试、所有相关
Cargo.toml/Cargo.lock、patches/、scripts/、根许可证和 README，以及源码校验元数据。
rebuild-locks/horizon-Cargo.lock 是打包可执行客户端实际使用的集成锁文件。
native-source-manifest.json 记录成功构建时每个原生源码、相关补丁与 staging 脚本的
SHA-256 和整体 nativeSourceSha256；打包时已逐项核对。

research/ 已附有 Horizon、LiquidBounce 与 Horizon-Mixins 三份固定提交的原始源码
ZIP。source-baselines.json 记录其完整提交、归档根目录、SHA-256 和备用 HTTPS 下载
地址。恢复工具优先验证附带归档的哈希、ZIP CRC、提交与路径，再创建本目录自己的
upstream/；只有归档缺失时才下载固定版本。已有归档或源基准不符时会报错并保留原
文件。包内没有复制构建缓存、node_modules、生成的 dist、Mojang 游戏资源或游戏
JAR，也不包含用户数据。上游原始源归档按原样保存，其中包括项目自带的构建工具。

需要 Windows x64、PowerShell 7、Python 3.11+、Git、rustup 管理的 stable 工具链
（Rust 1.95 或更新，并安装 rustfmt component）、带 x64 C++ 工具的 Visual Studio
Build Tools，以及 Node.js 20.19.0 或更新和 npm。Horizon 构建使用 cargo +stable，
CEF helper 使用当前 cargo 工具链；可在运行重建脚本前设置
`$env:RUSTUP_TOOLCHAIN = 'stable'`，使二者使用兼容版本。CMake 和 Ninja 由构建脚本
自动下载，无须预装。主题原锁文件与 package.json 定义依赖版本。
构建仍需访问 Cargo/npm 包仓库、CEF 发行源及构建工具下载源；三个源基准
可从包内归档直接恢复。这里提供的是锁定来源与依赖的源码重建；不同编译工具的二进制字节一致性
没有作为保证。

在此 source/ 目录运行：

```powershell
python .\Restore-Source-Baselines.py --verify-only
pwsh -NoProfile -File .\Rebuild-LiquidBounce.ps1
```

Rebuild-LiquidBounce.ps1 会恢复上游源基准、构建原版主题、创建独立 Horizon staging、
放入记录的集成 Cargo.lock，再调用随包的构建脚本。CEF helper、CEF 运行文件与游戏
可执行文件生成到本目录 build/staging/horizon/target/debug/。本流程不启动游戏，也不
准备或复制游戏资源。游戏运行仍须由使用者显式提供合法准备的 26.3 resources。

本包的启动器使用外层 runtime/ 中的已验收文件。重新构建后请对新客户端重新验收，
不要把原可执行文件的验收结果当成新构建的结果。模块范围见外层包 README；完整
236 模块移植仍未完成。
