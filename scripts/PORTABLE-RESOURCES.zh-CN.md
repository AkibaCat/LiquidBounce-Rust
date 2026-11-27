# 可迁移的 Minecraft 26.3 资源

原生游戏使用 `--resources <目录>`，直接读取 `assets/minecraft/`、`data/` 等展开文件。只提供 Minecraft 的 `assets/objects/` 不能补齐游戏的音效、字体和标题全景，因此本地发布包同时保留哈希对象、完整索引和全部索引资源的展开文件。

`prepare-portable-assets.py` 读取已有 Horizon prepared resources 和版本元数据，逐条校验索引引用的 SHA-1 与大小，生成全新的资源暂存目录。默认只读取本地缓存；加 `--download` 时，仅从 Mojang 官方索引及对象地址补齐缺少的文件。版本元数据缺少时，会按脚本固定的 26.3 元数据 SHA-1 获取并核验。脚本不会复制启动器账户、游戏世界或整个 Minecraft 安装目录。

```powershell
python scripts/prepare-portable-assets.py --prepared build/resources/26.3 --output build/portable-resources/26.3 --download
```

也可以指定现有同版本资产索引与缓存。Mojang 可能在同一个索引 ID 下更新翻译，显式本地索引会保留原字节及哈希，并在 `portable-assets.json` 中记录与版本元数据的差异。

```powershell
python scripts/prepare-portable-assets.py --prepared build/resources/26.3 --asset-index "$env:USERPROFILE/.gradle/caches/fabric-loom/assets/indexes/26.3-34.json" --source-assets "$env:USERPROFILE/.gradle/caches/fabric-loom/assets" --output build/portable-resources/26.3
```

输出目录结构：

```text
resources/26.3/
  horizon-resources.json
  asset-index.json
  portable-assets.json
  assets/
    indexes/34.json
    objects/<前两位 SHA-1>/<完整 SHA-1>
    minecraft/sounds.json
    minecraft/sounds/...
    minecraft/font/...
    minecraft/lang/...
    minecraft/resourcepacks/...
    minecraft/textures/gui/title/background/...
    pack.mcmeta
  data/...
  worldgen/...
```

对现有发布包先生成资源同步计划，检查后再加 `--apply`。该操作逐文件原子替换，备份写到工程 `build/` 下，失败则回滚。`user-data` 不会被枚举、读取或改写。资源改变后会移除可重建的 `horizon-data.bin` 与 `horizon-sprites.cache`。

```powershell
python scripts/sync-portable-resources.py --resources build/portable-resources/26.3 --package ../ReleasesPack/liquidbounce-horizon-263-native-20261127
python scripts/sync-portable-resources.py --resources build/portable-resources/26.3 --package ../ReleasesPack/liquidbounce-horizon-263-native-20261127 --apply
```

`metadata/resources-manifest.json` 记录资源逐文件 SHA-256，`metadata/portable-assets.json` 记录资产索引身份、对象数量、体积和引用校验结果。游戏程序的安全刷新脚本 `refresh-liquidbounce-preview.py` 会保留这组资源元数据，并重新生成整个包的文件清单。

原生启动器与 PowerShell 入口默认使用同包的 `resources/26.3`，发布包移动到其他位置后也无需原工程或 `.minecraft` 路径。`Package-LiquidBounce-Launcher.ps1` 优先选择完整的 `build/portable-resources/26.3`；也可显式传入 `-Resources`。

游戏资源、对象缓存和生成的发布包不属于 GPL 源码提交，应保留在忽略的 `build/` 或本地 `ReleasesPack/` 中。源码仓库分发准备脚本、许可证和原生代码。
