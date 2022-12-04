# cursinator

CLI tool for Installing/Download and managing addons/mods from CurseForge.

Useful for e.g. building modpacks for alternative launchers like SKCraft Launcher, or just installing some mods.

State: WIP, all features mentioned in the README do work. Few side functions and doc to be improved.

# Features

- Install addons  
- List addons  
- Release/Beta/Alpha channel mode for addons for e.g. auto-update  
- List/Show updates/files/versions of addon or all addons  
- Update addon or all addons  
- Remove/Purge addon  
- Create .url.txt for downloaded files  
- Generate CurseForge modpack manifest.json from template  

# Install

Build with API keys

```console
CURSEFORGE_API_KEY='...' cargo install -f --git https://github.com/CursedOnes/cursinator
```

Build without API keys, the API keys must then be supplied at runtime, via CURSEFORGE_API_KEY or inside repo.conf

```console
CURSEFORGE_API_KEY= cargo install -f --git https://github.com/CursedOnes/cursinator
```

# Example

```console
user:mods$ # Initialize Repo
user:mods$ cursinator init -g 1.16.x

user:mods$ # Install Addons
user:mods$ cursinator install jei
Install: jei (jei-1.16.5-7.6.4.90.jar)
Write repo json
user:mods$ cursinator install 267602
Install: ctm (CTM-MC1.16.1-1.1.2.6.jar)
Write repo json

user:mods$ # Install explicit file
user:mods$ cursinator install silents-gems=3.7.10
Install: silent-lib (silent-lib-1.16.3-4.9.6.jar)
Install: silents-gems (SilentGems-1.16.3-3.7.10+113.jar)
Write repo json

user:mods$ # List installed
user:mods$ cursinator list
ctm: ConnectedTexturesMod:  @BETA   
jei: Just Enough Items (JEI):  @BETA   
silent-lib: Silent Lib (silentlib):  @RELEASE
silents-gems: Silent's Gems:  @RELEASE

user:mods$ # List updates
user:mods$ cursinator updates
silents-gems: Silent's Gems RELEASE @RELEASE

user:mods$ # List updates for specific mod
user:mods$ cursinator updates silent
error: Ambiguous matches for installed addon
error: 	silent-lib
error: 	silents-gems
user:mods$ cursinator updates gems
RELEASE:   SilentGems-1.16.3-3.7.14.jar
RELEASE:   SilentGems-1.16.3-3.7.13.jar
RELEASE:   SilentGems-1.16.3-3.7.12+115.jar
RELEASE:   SilentGems-1.16.3-3.7.11+114.jar
INSTALLED: SilentGems-1.16.3-3.7.10+113.jar
...

user:mods$ # Update Addon
user:mods$ cursinator update silents-gem
Install: silents-gems (SilentGems-1.16.3-3.7.14.jar)
Remove previous version: SilentGems-1.16.3-3.7.10+113.jar
Write repo json

user:mods$ # Update All
user:mods$ cursinator update-all

user:mods$ # Remove Addon
user:mods$ cursinator purge silents-gems
Purging: silents-gems
Write repo json

user:mods$ # Remove unused deps
user:mods$ cursinator autoremove
Autoremove: silent-lib
Write repo json
```

# TODO

- [ ] Improve CLI help  
- [ ] Support search for addons other than Minecraft Mods  
- [ ] Improved regex for e.g. game version filter  
