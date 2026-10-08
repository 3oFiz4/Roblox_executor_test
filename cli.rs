use std::fs;
use std::io::{self, Write};
use std::path::Path;
use crate::dll_lib;
use colored::{Colorize, CustomColor};

/// Orange color helper (RGB: 255, 165, 0).
fn orange<T: std::fmt::Display>(text: T) -> colored::ColoredString {
    text.to_string().custom_color(CustomColor::new(255, 165, 0))
}

/// Attach to the DLL with proc. checking and clear status output.
pub fn attach_dll() -> Result<(), String> {
    println!("{}", "[*] Attempting to attach DLL...".yellow());

    let roblox_active = dll_lib::is_roblox_running();
    if !roblox_active {
        println!("{}", orange("[!] Warning: Roblox proc. ('RobloxPlayerBeta.exe') was not detected."));
        println!("{}", orange("Are you sure Roblox is running? so injection can succeed."));
    } else {
        println!("{}", "[+] Roblox proc. detected.".green());
    }

    match dll_lib::attach() {
        Ok(()) => {
            println!("{}", "[+] Attachment success.".green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("[-] Attach failed: {}", e).red());
            Err(e)
        }
    }
}

/// Execute a Lua script file from disk.
pub fn execute_file(file_path: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    if !path.exists() {
        let err = format!("Script file does not exist: {}", file_path);
        eprintln!("{}", format!("[-] {}", err).red());
        return Err(err);
    }

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let err = format!("Failed to read '{}': {}", file_path, e);
            eprintln!("{}", format!("[-] {}", err).red());
            return Err(err);
        }
    };

    if content.trim().is_empty() {
        println!("{}", orange(format!("[!] Warning: Script file '{}' is empty.", file_path)));
    }

    if !dll_lib::is_attached() {
        println!("{}", "[!] Notice: DLL does not currently report as attached.".cyan());
        println!("{}", "    If the script fails to run, execute 'attach' first.".cyan());
    }

    println!(
        "{}",
        format!(
            "[*] Executing script '{}' ({} bytes, {} lines)...",
            file_path,
            content.len(),
            content.lines().count()
        ).yellow()
    );

    match dll_lib::execute(&content) {
        Ok(()) => {
            println!("{}", format!("[+] Script '{}' execution success.", file_path).green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("[-] Script execution failed: {}", e).red());
            Err(e)
        }
    }
}

/// Execute raw Lua code directly from a string.
pub fn execute_string(code: &str) -> Result<(), String> {
    if code.trim().is_empty() {
        let err = "Cannot execute empty script.".to_string();
        eprintln!("{}", format!("[-] {}", err).red());
        return Err(err);
    }

    if !dll_lib::is_attached() {
        println!("{}", "[!] Notice: DLL does not currently report as attached.".cyan());
        println!("{}", "    If the script fails to run, execute 'attach' first.".cyan());
    }

    println!("{}", format!("[*] Executing raw script snippet ({} bytes)...", code.len()).yellow());

    match dll_lib::execute(code) {
        Ok(()) => {
            println!("{}", "[+] Script snippet execution success.".green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("[-] Script execution failed: {}", e).red());
            Err(e)
        }
    }
}

/// Show Roblox process and DLL attachment status.
pub fn show_status() {
    let roblox_running = dll_lib::is_roblox_running();
    let attached = dll_lib::is_attached();

    println!("{}", "  Roblox Executor Status".cyan());
    println!("{}", "==================================================".cyan());
    println!(
        "  Roblox Daemon: {}",
        if roblox_running {
            "[+] Running (detected)".green()
        } else {
            "[-] Not running".red()
        }
    );
    println!(
        "  DLL Attached:   {}",
        if attached {
            "[+] 1 (Attached)".green()
        } else {
            "[-] 0 (Not attached)".red()
        }
    );
    println!("{}", "==================================================".cyan());
}

/// Display help information.
pub fn show_help() {
    println!("{}", "CliType (CLI)".cyan());
    println!();
    println!("{}", "USAGE:".cyan());
    println!("  cli [COMMAND] [ARGS]");
    println!("  cli <command1> <command2> ...   (Supports chaining, e.g. 'attach exe script.lua')");
    println!();
    println!("{}", "COMMANDS:".cyan());
    println!("  attach, -a, --attach            Attach the DLL to the running Roblox process");
    println!("  exe <file>, exec, run, -f       Read and execute a Lua script file");
    println!("  eval <code>, raw <code>         Execute inline Lua script code");
    println!("  status, info, is_attached       Check Roblox process and DLL attach status");
    println!("  repl, interactive, -i           Start interactive command shell");
    println!("  help, -h, --help                Show this help screen");
    println!();
    println!("{}", "EXAMPLES:".cyan());
    println!("  cli attach                      # Attach DLL to Roblox");
    println!("  cli exe myscript.lua            # Execute script file");
    println!("  cli attach exe myscript.lua     # Attach first, then execute script");
    println!("  cli eval \"print('Hello!')\"      # Execute inline Lua snippet");
    println!("  cli status                      # Inspect status");
    println!("  cli                             # Start interactive REPL");
    println!();
}

/// Split an input line into tokens, respecting single and double quotes.
pub fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in input.chars() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Run commands parsed from a slice of argument strings.
pub fn run_cli(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return run_interactive();
    }

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "help" | "-h" | "--help" | "/?" => {
                show_help();
                i += 1;
            }
            "attach" | "att" | "a" | "-a" | "--attach" => {
                attach_dll()?;
                i += 1;
            }
            "exe" | "exec" | "e" | "execute" | "-e" | "--exe" | "-f" | "--file" => {
                if i + 1 >= args.len() {
                    let err = format!("Missing script file path for '{}' command. Usage: exe <file_path>", arg);
                    eprintln!("{}", format!("[-] {}", err).red());
                    return Err(err);
                }
                let file_path = &args[i + 1];
                execute_file(file_path)?;
                i += 2;
            }
            "eval" | "raw" | "run" | "r" => {
                if i + 1 >= args.len() {
                    let err = format!("Missing code string for '{}' command. Usage: eval <code>", arg);
                    eprintln!("{}", format!("[-] {}", err).red());
                    return Err(err);
                }
                // Join all remaining arguments or just the next argument
                let code = args[i + 1..].join(" ");
                execute_string(&code)?;
                break;
            }
            "status" | "s" | "info" | "is_attached" | "isattached" | "--status" => {
                show_status();
                i += 1;
            }
            "repl" | "interactive" | "-i" | "--interactive" => {
                return run_interactive();
            }
            // If argument is a path ending in .lua or .txt, treat as direct script file execution
            other if other.ends_with(".lua") || other.ends_with(".txt") => {
                execute_file(other)?;
                i += 1;
            }
            unknown => {
                let err = format!("Unknown command: '{}'. Type 'help' for usage.", unknown);
                eprintln!("{}", format!("[-] {}", err).red());
                return Err(err);
            }
        }
    }

    Ok(())
}

/// Run an interactive REPL shell.
pub fn run_interactive() -> Result<(), String> {
    println!("{}", "CliType (RBX Executor CLI)".cyan());
    println!("{}", "Type 'help' for available commands, 'exit' or 'quit' to quit.".cyan());
    println!();
    show_status();
    println!();

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("{}", "clitype> ".cyan());
        if let Err(e) = io::stdout().flush() {
            eprintln!("{}", format!("[-] Flush error: {}", e).red());
        }

        input.clear();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF reached (Ctrl+D or Ctrl+Z)
                println!();
                println!("{}", "[*] Exiting.".yellow());
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == "exit" || trimmed == "quit" || trimmed == "q" {
                    println!("{}", "[*] Goodbye!".yellow());
                    break;
                }

                if trimmed == "cls" || trimmed == "clear" {
                    print!("\x1B[2J\x1B[1;1H");
                    let _ = io::stdout().flush();
                    continue;
                }

                let tokens = split_args(trimmed);
                if tokens.is_empty() {
                    continue;
                }

                // Execute command
                if let Err(e) = execute_repl_command(&tokens) {
                    eprintln!("{}", format!("[-] Command failed: {}", e).red());
                }
                println!();
            }
            Err(e) => {
                eprintln!("{}", format!("[-] Error reading input: {}", e).red());
                break;
            }
        }
    }

    Ok(())
}

fn execute_repl_command(tokens: &[String]) -> Result<(), String> {
    let cmd = tokens[0].to_lowercase();
    match cmd.as_str() {
        "help" | "?" => {
            show_help();
            Ok(())
        }
        "attach" => {
            attach_dll()
        }
        "exe" | "exec" | "execute" | "run" => {
            if tokens.len() < 2 {
                eprintln!("{}", "[-] Missing file path. Usage: exe <file_path>".red());
                return Err("Missing file path".to_string());
            }
            execute_file(&tokens[1])
        }
        "eval" | "raw" => {
            if tokens.len() < 2 {
                eprintln!("{}", "[-] Missing code. Usage: eval <code>".red());
                return Err("Missing code".to_string());
            }
            let code = tokens[1..].join(" ");
            execute_string(&code)
        }
        "status" | "info" | "is_attached" | "isattached" => {
            show_status();
            Ok(())
        }
        other if other.ends_with(".lua") || other.ends_with(".txt") => {
            execute_file(other)
        }
        unknown => {
            eprintln!("{}", format!("[-] Unknown command: '{}'. Type 'help' for command list.", unknown).red());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_args() {
        let args = split_args("exe script.lua");
        assert_eq!(args, vec!["exe", "script.lua"]);

        let args_quoted = split_args("exe \"my script with spaces.lua\"");
        assert_eq!(args_quoted, vec!["exe", "my script with spaces.lua"]);

        let args_chained = split_args("attach exe 'test file.lua'");
        assert_eq!(args_chained, vec!["attach", "exe", "test file.lua"]);
    }

    #[test]
    fn test_file_not_found() {
        let result = execute_file("non_existent_file_12345.lua");
        assert!(result.is_err());
    }

    #[test]
    fn test_help_no_crash() {
        show_help();
    }

    #[test]
    fn test_status_no_crash() {
        show_status();
    }
}
