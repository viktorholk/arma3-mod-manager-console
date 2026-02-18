<div align="center">

# Arma 3 Mod Manager Console

A lightweight terminal-based mod manager for Arma 3 on **Linux** and **macOS**.

Enable, disable, and launch mods without the official launcher.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/viktorholk/arma3-mod-manager-console)](https://github.com/viktorholk/arma3-mod-manager-console/releases/latest)

[Website](https://viktorholk.github.io/arma3-mod-manager-console) · [Download](https://github.com/viktorholk/arma3-mod-manager-console/releases/latest) · [Issues](https://github.com/viktorholk/arma3-mod-manager-console/issues)

<img src="https://github.com/user-attachments/assets/f5f58180-e5f4-4442-a448-c60f81df907d" alt="Demo" width="700" />

</div>

## Features

- **Mod toggling** — enable/disable mods with a keypress
- **Mod presets** — save and switch named mod loadouts
- **Dependency checking** — detect missing mod dependencies
- **Direct launch** — start Arma 3 from the manager
- **Setup wizard** — guided first-run configuration
- **CDLC support** — manage Creator DLC alongside mods
- **Custom mods** — load offline mods from a local folder

## Requirements

- Arma 3 installed via Steam
- Rust & Cargo (if building from source)

## Installation

### Pre-built binary

Download the latest release for your platform from [releases](https://github.com/viktorholk/arma3-mod-manager-console/releases/latest):

| Platform | File |
|---|---|
| macOS Apple Silicon (M1+) | `arma3-mod-manager-console-*-aarch64-apple-darwin-release.zip` |
| macOS Intel | `arma3-mod-manager-console-*-x86_64-apple-darwin-release.zip` |
| Linux x86_64 | `arma3-mod-manager-console-*-x86_64-unknown-linux-gnu-release.zip` |
| Linux ARM | `arma3-mod-manager-console-*-aarch64-unknown-linux-gnu-release.zip` |

Unzip, make executable (`chmod +x arma3-mod-manager-console`), and run.

<details>
<summary>macOS security warning</summary>

On macOS, you may be greeted with a security warning.
Go to **Settings > Privacy & Security > Security** and press **Open Anyway**.

![image](https://github.com/user-attachments/assets/966592ac-b40a-439e-b793-70fc42070ccd)

![image](https://github.com/user-attachments/assets/6d58efce-6dff-41f9-b790-7839c2a15a36)

</details>

### Build from source

```sh
git clone https://github.com/viktorholk/arma3-mod-manager-console.git
cd arma3-mod-manager-console
cargo run --release
```

## Usage

### Controls

| Action | Keys |
|---|---|
| Navigate | `W` `S` / `K` `J` / `↑` `↓` |
| Toggle mod | `Space` |
| Cycle presets | `Tab` / `Shift+Tab` |
| Preset manager | `T` |
| Launch Arma 3 | `P` |
| Search | `/` |
| Check dependencies | `D` |
| Quit | `Q` |

### Presets

Presets let you save named mod selections and switch between them without manually toggling mods each time.

## Configuration

The config file is located at:

```
~/.config/arma3-mod-launcher-console/config.json
```

```json
{
  "game_path": "/path/to/Steam/steamapps/common/Arma 3",
  "workshop_path": "/path/to/Steam/steamapps/workshop/content/107410",
  "custom_mods_path": "/path/to/custom-mods",
  "executable_name": "arma3",
  "enabled_mods": [],
  "default_args": "-noSplash -skipIntro -world=empty",
  "presets": {
    "Default": []
  },
  "active_preset": "Default"
}
```

If the application cannot resolve the correct paths, you can edit them here. The `executable_name` field allows you to specify a different Arma 3 executable name:

- On macOS: without the `.app` extension (e.g., `arma3`)
- On Linux: the actual executable name (e.g., `arma3_x64`)

### Custom mods

Place your mods in the custom mods folder. The folder is created alongside the config file.

## Troubleshooting

### InvalidPath error

If you see `Error: InvalidPath(...)` on startup, the Steam paths in your config are incorrect.

1. Open the config file (see above)
2. Set `game_path` and `workshop_path` to the correct Steam directories:
   - macOS: `~/Library/Application Support/Steam/steamapps/...`
   - Linux: `~/.local/share/Steam/steamapps/...`
3. Save and rerun

### Mod compatibility

Not all Arma 3 mods work on macOS or Linux. Mods that require Windows .DLL files (ACE, TFAR/ACRE, Blastcore, etc.) are not compatible. Most content mods (maps, units, vehicles) work fine.

## License

[MIT](LICENSE)
