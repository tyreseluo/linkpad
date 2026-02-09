# Linkpad

Linkpad is a desktop proxy client built with Makepad (`makepad-components` + `makepad-shell`) and a Rust core crate (`linkpad-core`) powered by the Mihomo kernel.

## Current Scope

- macOS-first implementation
- Architecture prepared for future Windows/Linux/mobile expansion
- Desktop kernel: Mihomo

## Implemented Features

- Profile import from URL
- Clash YAML parsing, including Base64 subscription payload support
- Profile lifecycle: import, activate, refresh, delete
- Profile persistence across restarts
- Proxy Groups page
- Mode switching: `Rule` / `Global` / `Direct`
- Per-group proxy selection (applied to Mihomo controller)
- Latency test with progressive updates
- Timeout rendering
- Locate current selected proxy in group list
- Rules page
- Search + filter (`All`, `DOMAIN`, `IP-CIDR`, `PROCESS-NAME`)
- Progressive lazy loading
- Settings page
- Language (`English`, `简体中文`) + i18n switching
- Theme (`Light`, `Dark`, `System`) with persistent state
- System settings (`System Proxy`, `Auto Launch`, `Silent Start`, `Run in background on close`)
- Clash settings (`mixed-port`, core version, upgrade, restart)
- Tray integration
- Outbound mode submenu
- Active profile submenu
- System proxy toggle
- Exit
- Notification system
- Core runtime integration
- Start/stop/restart kernel runtime
- System proxy management (macOS)
- Kernel binary upgrade/check flow

## Project Structure

```text
.
├── crates/
│   └── core/                  # linkpad-core: runtime, parser, proxy/kernel logic
├── linkpad/
│   ├── src/
│   │   ├── app.rs             # app shell + global UI orchestration
│   │   ├── tray.rs            # tray menu integration
│   │   ├── views/             # page/view logic split by domain
│   │   │   ├── profiles.rs
│   │   │   ├── proxy_groups.rs
│   │   │   ├── rules.rs
│   │   │   └── settings.rs
│   │   ├── store/             # persistence modules
│   │   │   ├── profile_store.rs
│   │   │   └── settings_store.rs
│   │   ├── i18n/              # localization resources
│   │   └── ui/                # Makepad live UI definitions
│   └── assets/
└── Cargo.toml
```

## Run Locally

```bash
cargo run -p linkpad
```

## Runtime/Kernel Notes

Linkpad resolves `mihomo` from multiple locations with this rough priority:

- `LINKPAD_MIHOMO_PATH`
- `~/Library/Application Support/linkpad/bin/mihomo` (macOS, `UPGRADE` writes here)
- Bundled app resource (on macOS, e.g. `Linkpad.app/Contents/Resources/linkpad/bin/mihomo`)
- System `PATH`

Release pipeline (`.github/workflows/release.yml`) runs `scripts/prepare-bundled-mihomo.sh`:

- Uses only Linkpad internal kernel path under `linkpad/resources/bin/` (never reuses local `mihomo`)
- Downloads from Mihomo releases only when internal kernel is missing
- Uses versioned bundled filename (for example `mihomo-darwin-arm64-v1.19.20`)
- Resolves the release asset by target OS/arch (auto-detected from build target or host)

Useful environment variables:

- `LINKPAD_MIHOMO_PATH`: override Mihomo binary path
- `LINKPAD_BUNDLED_MIHOMO_VERSION`: pin packaged Mihomo version (for example `v1.19.19`)
- `LINKPAD_BUNDLED_MIHOMO_OS`: force target OS (`darwin` / `windows` / `linux` / `android`)
- `LINKPAD_BUNDLED_MIHOMO_ARCH`: force target arch (`arm64` / `amd64` / `386` / `armv7`)
- `LINKPAD_BUNDLED_MIHOMO_TARGET`: force target triple (for example `aarch64-apple-darwin`)
- `LINKPAD_BUNDLED_MIHOMO_REFRESH=1`: force refresh the bundled kernel before packaging
- `LINKPAD_BUNDLED_MIHOMO_STRICT=1`: strict mode, fail packaging when internal kernel is missing
- `LINKPAD_BUNDLED_MIHOMO_TEST_MODE=1`: test mode, disable network download and use local directory
- `LINKPAD_BUNDLED_MIHOMO_TEST_DIR=/path/to/dir`: local kernel directory for test mode
- `LINKPAD_GITHUB_TOKEN`: increase GitHub API rate limit for kernel upgrade
- `RUST_LOG=linkpad=info,linkpad_core=info`: enable runtime logs

## Persistent Data

Stored under config directory (macOS usually `~/Library/Application Support/linkpad`):

- `settings.json`
- `profiles.json`

## Known Limitations

- System proxy manager is currently implemented for macOS only
- Startup item management is currently implemented for macOS only
- TUN mode is not integrated yet
- App menu is reserved as placeholder (tray is active)
