# Roblox_executor_test
🟢 Working (last tested: 7, Oct 2026)

A TUI made in Rust with for simple roblox (sUNC 85%) hack. Probably the **first** Roblox executor in TUI/CLI form lol. I haven't think of a beter name of the executor yet... Right now its called CliType 

READ ME: **Also please understand to NOT abuse this script. I know hacking is fun. I mean, I am addicted to it. But to ever ever use this script for the sole purpose of making people gameplay worse are not cool. I personally use this to automate a coin farm on some roblox game, almost never use it for Aimbot, or something. Karma is real. Trust me.**

https://github.com/user-attachments/assets/570b3cb9-47e9-4d82-b482-f99be529f4fd

<img width="964" height="1068" alt="image" src="https://github.com/user-attachments/assets/8e2c91a2-34ce-4e8e-8152-ce2bba97b9e6" />


## Notice

There is some change to the `.dll`. The `.dll` is indeed modified, although it remains original, except that I add a credit to myself within the .dll. You can do binary comparison of the original `WRD API` with `rbx-1.dll`. Specifically, I changed this part right here:
```
Look up to 0x204B80–0x204BC0 (RVA 0x205780–0x2057C0 in the .rdata section)

00204B70  90 3c 1e 80 01 00 00 00 30 57 20 80 01 00 00 00   .<......0W .....
00204B80  50 3c 1e 80 01 00 00 00 f0 8e 02 80 01 00 00 00   P<..............
00204B90  e0 8e 02 80 01 00 00 00 10 3c 1e 80 01 00 00 00   .........<......
00204BA0  90 74 02 80 01 00 00 00 57 52 44 20 41 50 49 20   .t......WRD API
00204BB0  52 65 61 64 79 20 28 45 78 65 63 75 74 6f 72 20   Ready (Executor
00204BC0  62 79 3a 20 33 6f 66 69 7a 34 29 52 6f 62 6c 6f   by: 3ofiz4)Roblo
00204BD0  78 50 6c 61 79 65 72 42 65 74 61 2e 65 78 65 52   xPlayerBeta.exeR
00204BE0  6f 62 6c 6f 78 73 72 63 5c 67 61 6d 65 2e 72 73   obloxsrc\game.rs
00204BF0  e0 57 20 80 01 00 00 00                            .W ....
```

Original hex: `57 65 41 72 65 44 65 76 73 20 41 50 49 20 77 72 61 70 70 65 72 20 6C 6F 61 64 65 64`

Replaced hex: `57 52 44 20 41 50 49 20 52 65 61 64 79 20 28 45 78 65 63 75 74 6F 72 20 62 79 3A 20 33 6F 66 69 7A 34 29`

Target Replacement String: `WRD API Ready (Executor by: 3ofiz4)` (35 bytes)

<img width="1530" height="1080" alt="image" src="https://github.com/user-attachments/assets/d83e9ea4-53c0-4e07-8aa6-a31824c206e4" />

## Installation
1. Git clone this. OR, install via Release.
2. Run `cargo run`, or do the build version by yourself, and ensure that the `.dll` is sibling to the executable.
3. Enter any roblox game.
4. Do `:attach`
5. Wait until your window glitch.
6. Add your script then do `:exe` to execute the script

## TUI Control/Command
### Control
The executor imitates Vim motion. Please take a read of how Vim motion works. I'll add lay-peeps version soon.
1. Movement (hjkl; w = word; b = back), not everything is developed currently.
2. Search
3. Command. Click `:`.

### Command list 
```
:attach          |          Attach the DLL binary file to Roblox
:is_attach       |          Check if you actually connected or not
:exe             |          Execute the script in your text currently.
```

## CLI Command
If you think the TUI is bugged/glitched (sorry :P...), this is a another workaround, but you won't see any script placeholder. So it expects you to have VSCode right beside this CLI.
### Command list
```
COMMANDS:
  attach, -a, --attach            Attach the DLL to the running Roblox process
  exe <file>, exec, -f       Read and execute a Lua script file
  run, eval <code>, raw <code>         Execute inline Lua script code
  status, info, is_attached       Check Roblox process and DLL attach status
  repl, interactive, -i           Start interactive command shell
  help, -h, --help                Show this help screen
EXAMPLES:
  cli attach                      # Attach DLL to Roblox
  cli exe myscript.lua            # Execute script file  cli attach exe myscript.lua     # Attach first, then execute script
  cli eval "print('Hello!')"      # Execute inline Lua snippet
  cli status                      # Inspect status
  cli                             # Start interactive REPL
```

## Coming soon
1. Script collections.
2. Idk.. I'll think more of it.

---
rawr
