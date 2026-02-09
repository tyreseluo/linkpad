# Linkpad

Linkpad 是一个基于 Makepad（`makepad-components` + `makepad-shell`）和 Rust Core（`linkpad-core`）实现的桌面代理客户端，内核使用 Mihomo。

## 当前阶段

- 目前优先支持 macOS
- 架构上已经为 Windows / Linux / 移动端扩展预留
- 桌面端代理内核：Mihomo

## 已实现能力

- Profile URL 导入
- 支持 Clash YAML 解析，包含 Base64 订阅内容解析
- Profile 管理流程：导入、激活、刷新、删除
- Profile 持久化（重启后恢复）
- Proxy Groups 页面
- 模式切换：`Rule` / `Global` / `Direct`
- 每个 Group 内可选 Proxy，并真实下发到 Mihomo Controller
- 延迟测速（渐进更新）
- 超时状态展示
- 定位当前已选 Proxy
- Rules 页面
- 搜索与分类过滤（`All` / `DOMAIN` / `IP-CIDR` / `PROCESS-NAME`）
- 规则渐进懒加载
- Settings 页面
- 语言切换（English / 简体中文）与 i18n 联动
- 主题切换（Light / Dark / System）与持久化
- 系统设置（System Proxy / Auto Launch / Silent Start / 关闭时后台运行）
- Clash 设置（mixed-port、内核版本、升级、重启）
- 托盘能力
- 出站模式子菜单
- Profile 子菜单（激活项勾选）
- System Proxy 开关
- Exit 退出
- 全局通知系统
- Core 运行时接入
- 内核启动 / 停止 / 重启
- macOS 系统代理管理
- 内核二进制升级与校验流程

## 项目结构

```text
.
├── crates/
│   └── core/                  # linkpad-core：运行时、解析器、代理/内核逻辑
├── linkpad/
│   ├── src/
│   │   ├── app.rs             # 应用壳层与全局 UI 编排
│   │   ├── tray.rs            # 托盘菜单集成
│   │   ├── views/             # 页面逻辑按领域拆分
│   │   │   ├── profiles.rs
│   │   │   ├── proxy_groups.rs
│   │   │   ├── rules.rs
│   │   │   └── settings.rs
│   │   ├── store/             # 持久化模块
│   │   │   ├── profile_store.rs
│   │   │   └── settings_store.rs
│   │   ├── i18n/              # 多语言资源
│   │   └── ui/                # Makepad live UI 定义
│   └── assets/
└── Cargo.toml
```

## 本地运行

```bash
cargo run -p linkpad
```

## 内核运行说明

Linkpad 会在多个位置查找 `mihomo`，优先级大致如下：

- 直接设置 `LINKPAD_MIHOMO_PATH`
- `~/Library/Application Support/linkpad/bin/mihomo`（macOS，`Upgrade` 后会写入这里）
- 安装包内置资源（macOS 例如 `Linkpad.app/Contents/Resources/linkpad/bin/mihomo`）
- 系统 `PATH`

发布流程会在 `.github/workflows/release.yml` 中执行 `scripts/prepare-bundled-mihomo.sh`：

- 仅使用 Linkpad 内部路径 `linkpad/resources/bin/` 的内核（不会复用本机 `mihomo`）
- 如果内部内核不存在，则按目标平台/架构从 Mihomo release 下载并写入内部路径
- 内嵌产物使用版本化命名（例如 `mihomo-darwin-arm64-v1.19.20`）
- 目标平台可由打包环境自动识别，也可用环境变量强制指定

常用环境变量：

- `LINKPAD_MIHOMO_PATH`：指定 Mihomo 路径
- `LINKPAD_BUNDLED_MIHOMO_VERSION`：指定打包下载的 Mihomo 版本（如 `v1.19.19`）
- `LINKPAD_BUNDLED_MIHOMO_OS`：强制目标 OS（`darwin` / `windows` / `linux` / `android`）
- `LINKPAD_BUNDLED_MIHOMO_ARCH`：强制目标架构（`arm64` / `amd64` / `386` / `armv7`）
- `LINKPAD_BUNDLED_MIHOMO_TARGET`：强制目标 triple（如 `aarch64-apple-darwin`）
- `LINKPAD_BUNDLED_MIHOMO_REFRESH=1`：打包前强制重新准备内嵌内核
- `LINKPAD_BUNDLED_MIHOMO_STRICT=1`：严格模式，仅允许使用内部内核，缺失时直接失败
- `LINKPAD_BUNDLED_MIHOMO_TEST_MODE=1`：测试模式，不联网下载，改为从本地目录读取内核
- `LINKPAD_BUNDLED_MIHOMO_TEST_DIR=/path/to/dir`：测试模式下的本地内核目录
- `LINKPAD_GITHUB_TOKEN`：提高 GitHub API 限流阈值（用于升级）
- `RUST_LOG=linkpad=info,linkpad_core=info`：开启关键日志

## 持久化数据

配置目录下（macOS 一般是 `~/Library/Application Support/linkpad`）：

- `settings.json`
- `profiles.json`

## 已知限制

- System Proxy 目前只实现了 macOS
- 开机启动管理目前只实现了 macOS
- TUN 模式尚未接入
- App Menu 目前仅预留（Tray 已可用）
