# Arma3 Mod Manager Console

<p align="center">
  <img src="https://github.com/user-attachments/assets/f5f58180-e5f4-4442-a448-c60f81df907d" alt="animated" />
</p>

## Overview

Arma 3 Mod Manager Console is a lightweight tool that helps you enable, disable, and manage Arma 3 Steam Workshop mods and creator DLCs without needing the official Arma 3 Launcher. Designed for Linux and macOS, this console-based mod manager is perfect for players who want a simple and efficient way to control their mod list.

## Installation

### Requirements

- Arma 3 installed via Steam
- Rust & Cargo installed (if building from source)

### Download & Install
#### Download Pre-Built Binary

Downloading the latest Pre-Built UNIX executable from [releases](https://github.com/viktorholk/arma3-mod-manager-console/releases).

<details><summary>For MacOS</summary>

On MacOS, you may be greeted with a security warning.
Go to Settings > Privary & Security > Security
and press Open Anyway

![image](https://github.com/user-attachments/assets/966592ac-b40a-439e-b793-70fc42070ccd)


![image](https://github.com/user-attachments/assets/6d58efce-6dff-41f9-b790-7839c2a15a36)

</details>

#### Build from Source
````
git clone git@github.com:viktorholk/arma3-mod-manager-console.git
cd arma3-mod-manager-console
cargo run
````

## Config
The application creates a config file at `~/arma3-mod-launcher-console-config.json` which looks like this:

````
{
  "game_path": "/Users/user/Library/Application Support/Steam/steamapps/common/Arma 3",
  "workshop_path": "/Users/user/Library/Application Support/Steam/steamapps/workshop/content/107410",
  "custom_mods_path": "/Users/user/arma3-mod-manager-console-custom-mods",
  "enabled_mods": [
    
  ],
  "default_args": "-noSplash -skipIntro -world=empty"
}
````

If the application cannot resolve the correct paths, you can edit them here.

### Custom Mods

Simply move your mods into the custom mods folder. The folder will be alongside the config.

## Troubleshooting Guide (Fix Errors & Paths)

**Issue**: Running the console gives an error: 

`Error: InvalidPath("/Users/user/Library/Application Support/Steam/steamapps/workshop/content/107410")`

**Steps to Resolve**:
1. **Check Config File**: Verify `~/arma3-mod-manager-console-config.json` has the correct Steam path.
2. **Ensure Workshop Mods**: Confirm Arma 3 workshop mods are installed via Steam.
3. **Locate Steam Path**:
   - For MacOS check for `~/Library/Application Support/Steam`
   - For Linux check for  `~/.local/share/Steam`

**Adjust and test** the paths, then rerun the application.

## Issues
Need Help? [GitHub's issues tab](https://github.com/viktorholk/script-interactor/issues).

## License
Arma 3 Mod Manager Console is under the [MIT LICENSE](LICENSE).
