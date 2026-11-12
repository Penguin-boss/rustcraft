<div align="center">

# RustCraft

**A fast, open-source Minecraft launcher written in Rust.**

Instances, modpacks, Java runtimes and accounts — all in one place, with a native
interface rendered by [Freya](https://freyaui.dev).

</div>

---

> **About the name.** RustCraft is a rebranded fork of the open-source
> [OneLauncher / OneClient](https://github.com/Polyfrost/OneLauncher) project by
> Polyfrost. The code is the same GPL-3.0-only launcher; only the product branding
> has changed. See [Naming & compatibility](#naming--compatibility) for exactly what
> was renamed and what was kept on purpose.

## Contents

- [Features](#features)
- [Requirements](#requirements)
- [Building & running](#building--running)
- [Packaging](#packaging)
- [Project layout](#project-layout)
- [Where RustCraft stores its files](#where-rustcraft-stores-its-files)
- [Naming & compatibility](#naming--compatibility)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License & credits](#license--credits)

## Features

### Instances that stay in sync

- Create a **RustCraft instance** from the curated version catalogue, or a **custom
  instance** with the Minecraft version, mod loader and loader version you choose.
- Give each instance its own game folder, or **share one folder** between instances so
  worlds, options and packs carry over.
- Loader updates are applied for managed instances and proposed for custom ones.
- Per-instance **Java overrides**, game arguments, memory settings and graphics backend.

### Mods, packs and content

- Browse and install from **Modrinth** and **CurseForge** without leaving the launcher.
- Dependency resolution, duplicate detection, update checks and one-click removal of
  dependents.
- **Global packages** shared across every instance, plus optional per-instance opt-outs.
- Manage resource packs, shader packs, datapacks, worlds and screenshots from the same
  instance view.
- Import **`.mrpack`** files, CurseForge packs and exported instances.

### Bring your existing setup with you

- Import instances from **Prism Launcher, MultiMC, PolyMC and the Modrinth App**, as well
  as the legacy OneClient v1 data folder and vanilla Minecraft installations.
- Export an instance as a portable archive, or share it with a **share code / link**.
  Worlds are never included in shares.

### Accounts and Java

- A single **offline username**. RustCraft derives the vanilla offline UUID from it,
  stores it locally and never contacts Microsoft, Xbox Live or Mojang.
- Automatic Java discovery and a managed runtime store, with installers backed by
  Adoptium, Amazon Corretto, BellSoft Liberica, Microsoft, Mojang and Azul Zulu.
- Runtime repairs for missing natives, assets and libraries when a launch fails.

### Diagnostics that actually help

- Live log console with filtering and parsed output.
- Crash reports, suggested fixes and mod-problem detection.
- **Bisect mode** to narrow down which mod breaks a launch.
- Snapshot of the launcher, Java and hardware state for bug reports.

### Quality-of-life

- Desktop **shortcuts** for instances, plus share links and launch shortcuts handled through
  the retained `oneclient://` protocol registration.
- System tray / menu bar support and run-in-background mode.
- Light, dark and custom **themes**, with a theme editor.
- Translations, Discord rich presence and a built-in updater for packaged builds.
- Social features (chat, parties, world hosting) through Poly+ are compiled in but
  unreachable: they require a Microsoft session, which RustCraft does not have.

## Requirements

| | |
|---|---|
| **Rust** | 1.97 or newer (edition 2024) — install via [rustup](https://rustup.rs/) |
| **OS** | Windows 10+ (the release automation builds Windows x64 only) |
| **Linux extras** | C toolchain, `pkg-config`, and the GUI/graphics dev packages required by [Freya](https://book.freyaui.dev/setup.html) — only needed if you build the sources there yourself |
| **Other** | A Minecraft: Java Edition installation you are allowed to use; internet access for downloads |

## Building & running

```sh
# Debug build (data lives in a separate "OneClient-dev" folder)
cargo run -p oneclient_app

# Optimized build
cargo build --release -p oneclient_app
```

The binary is produced at `target/release/oneclient_app` (the internal binary name was
kept for compatibility). On Linux, `cargo run` needs the usual X11/Wayland and font
development packages installed.

Run the test suite with:

```sh
cargo test --workspace
```

Formatting and lints:

```sh
cargo fmt
cargo clippy --workspace --all-targets
```

## Packaging

Installers are produced with [**cargo-packager**](https://github.com/crabnebula-dev/cargo-packager).
Configuration lives in [`packages/oneclient_app/Cargo.toml`](./packages/oneclient_app/Cargo.toml)
under `[package.metadata.packager]`.

```sh
cargo install cargo-packager --locked

cargo build --release -p oneclient_app
cargo packager --release -p oneclient_app --formats nsis        # Windows x64
```

The release and nightly GitHub Actions workflows build **Windows x64 only** and publish an
NSIS installer (`RustCraft_<version>_x64-setup.exe`) plus the updater manifest. The
`cargo-packager` metadata still defines `app`, `dmg`, `deb` and `appimage` formats, and the
`deb` desktop-entry template plus `cargo-generate-rpm` metadata remain in
`packages/oneclient_app/distribution/linux/`, so a local macOS or Linux package can still be
produced by hand. Distribution recipes for AUR, COPR and Flatpak live in
[`distribution/`](./distribution); no workflow publishes them.

> **Note:** the packaging recipes and release workflows in this repository still reference
> the upstream Polyfrost project and its artifact names. They are kept so existing
> distribution channels and update metadata keep working.

### Versioning

The workspace shares a single version defined in the root [`Cargo.toml`](./Cargo.toml)
under `[workspace.package]`. Release notes come from [Knope](https://knope.tech) change
files in `.changeset/`:

```sh
knope document-change
```

## Project layout

RustCraft is a Cargo workspace. All crates live under `packages/`.

| Crate | Responsibility |
|---|---|
| `oneclient_app` | Freya desktop UI, actions, views, IPC and platform glue |
| `oneclient_core` | Launcher core: instances, settings, migrations, launches, diagnostics |
| `oneclient_cluster` | Instance ("cluster") records, settings profiles and on-disk files |
| `oneclient_content` | Mod providers (Modrinth / CurseForge), artifact store and bundles |
| `oneclient_mc` | Minecraft manifests, assets, libraries, natives and launch arguments |
| `oneclient_java` | Java discovery, probing and runtime installation |
| `oneclient_auth` | Offline local profile and `auth.json` storage (no Microsoft OAuth) |
| `oneclient_db` | SQLx / SQLite persistence layer and migrations |
| `oneclient_net` | HTTP transport: retries, throttling, caching and verified downloads |
| `oneclient_events` | Event bus for notifications, progress and prompts |
| `oneclient_discord` | Discord rich presence integration |
| `oneclient_polyplus` | Poly+ party / social / playtime integration (unavailable offline) |
| `oneclient_common` | Shared paths, constants, platform helpers and content types |
| `oneclient_macro` | UI helper macros |
| `polyio` | Async filesystem, hashing and archive utilities |

> Crate names still use the `oneclient_` prefix. Renaming them would touch every
> `use` statement, packaging recipe and database migration; the prefix is an internal
> detail and is intentionally left alone.

## Where RustCraft stores its files

RustCraft keeps its configuration, database and game data together:

| Platform | Default location |
|---|---|
| Windows | `%APPDATA%\Polyfrost\OneClient` |
| macOS | `~/Library/Application Support/Polyfrost/OneClient` |
| Linux | `~/.local/share/Polyfrost/OneClient` |

Inside that folder you will find:

| File / folder | Purpose |
|---|---|
| `settings.json` | Launcher settings |
| `auth.json` | The single offline profile (username and derived UUID) |
| `user_data.db` | Instances, packages, sessions and other SQLite state |
| `metadata/`, `java/`, `packages/` | Cached manifests, managed Java runtimes and the artifact store |

`auth.json` is intentionally tiny and contains nothing secret:

```json
{
  "profile": {
    "id": "b50ad385-829d-3141-a216-7e7d7539ba7f",
    "username": "Notch"
  }
}
```

The `id` is recalculated from `username` on load, so a hand-edited file can never drift
away from the UUID offline-mode servers use. Files written by older OneClient builds are
migrated in place: the default account (or an explicit offline account) becomes the
profile and every Microsoft token is dropped.

Debug builds use the `OneClient-dev` folder next to the release one, so development never
touches your real setup. The data folder can be moved from
**Settings → Launcher → Data folder**.

The `OneClient` folder name is deliberately unchanged: renaming it would orphan existing
installs, worlds and settings. It is the on-disk identity of the launcher, not its
display name.

## Naming & compatibility

This fork renames everything a player sees, and keeps everything a machine depends on.
That balance is what lets RustCraft build, update and read existing installs.

**Renamed to RustCraft**

- Window title, tray menu, onboarding and every in-app message / translation string
- Desktop entries, Linux/Flatpak display names, installer product name, Windows file
  description, Discord presence text and the `${launcher_name}` passed to Minecraft
- Docs, issue forms, CI workflow display names and crate descriptions

**Kept on purpose**

| Identifier | Why |
|---|---|
| `oneclient_*` crate and directory names | Internal structure; renaming means a workspace-wide refactor with no user benefit |
| `org.polyfrost.OneClient` bundle / app ID, IPC pipe name | Update identity and single-instance handling; changing it breaks in-place upgrades |
| `oneclient://` URL scheme | Existing shortcuts, share links and protocol registrations |
| `OneClient` data folder, `OneClient-dev` debug folder | Existing user data and settings |
| Upstream URLs (`github.com/Polyfrost/OneLauncher`, `polyfrost.org`) | Release downloads, changelog feed, updater endpoint and bug-report links; they still resolve to the upstream project |
| Asset names (`OneClient.icon`, `onelauncher_logo_512`, Flatpak IDs) | Files and store listings that are referenced by name |

If you later host RustCraft yourself, update `UPDATER_ENDPOINT`, `RELEASES_URL`,
`CHANGELOG_URL` and `REPORT_URL` in `packages/oneclient_app/src/constants.rs` and
`packages/oneclient_core/src/changelog.rs`, then adjust the workflows in `.github/`.

## Troubleshooting

**The server says I am not authenticated.** RustCraft only joins offline-mode
(`online-mode=false`) servers. For an online-mode server you need the official launcher,
because joins are authenticated by Mojang.

**My inventory or progress reset on a server.** The offline UUID is derived from the exact
username, so changing the name changes the player. Use the same spelling everywhere;
`Notch` and `notch` are different players. The UUID shown in
**Settings → Accounts** is exactly what offline-mode servers derive.

**My skin is missing.** Skins and capes need a Microsoft-authenticated Mojang profile, so
the wardrobe is unavailable in RustCraft. Servers and singleplayer still apply any skin
from the local `skins` folder or a resource pack.

**A version is missing.** Managed instance versions come from the RustCraft/OneClient
catalogue. If a version is not listed, create a custom instance with that Minecraft
version and loader instead.

**The game starts without a mod.** RustCraft disables content that disappeared from disk
and tells you in the instance's notices. Reinstall the package or restore the file, then
re-enable it.

**Where are the logs?** Open the instance's **Logs** tab, or use the log console in
Settings. Crash reports include the launcher, Java and hardware details.

**Reset everything.** Close RustCraft and rename (do not delete) the data folder listed
above; a fresh one is created on the next start.

## Contributing

Contributions are welcome. Read [`CONTRIBUTING.md`](./CONTRIBUTING.md) first, then:

1. Fork the repository and create a topic branch.
2. Make your change, add a change file with `knope document-change`, run `cargo fmt`,
   `cargo clippy` and `cargo test --workspace`.
3. Open a pull request describing player-visible behaviour.

## License & credits

RustCraft is distributed under the **GNU General Public License v3.0 only**. See
[`LICENSE`](./LICENSE) and [`ATTRIBUTION.md`](./ATTRIBUTION.md).

- Original project: **OneLauncher / OneClient** by
  [Polyfrost and contributors](https://github.com/Polyfrost/OneLauncher), GPL-3.0-only.
- UI framework: [Freya](https://freyaui.dev) and [Skia](https://skia.org/).
- Mod metadata: [Modrinth](https://modrinth.com) and [CurseForge](https://curseforge.com).
- Java runtimes: Adoptium, Amazon Corretto, BellSoft Liberica, Microsoft, Mojang, Azul Zulu.
- Branding images and logos remain the property of their respective owners; see
  `ATTRIBUTION.md` for the Polyfrost branding terms.

RustCraft is a community rebrand and is **not affiliated with or endorsed by Polyfrost,
Mojang Studios or Microsoft**. Minecraft is a trademark of Mojang Synergies AB.
