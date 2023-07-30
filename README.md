<h1 align="center">nu-plugin-highlight</h1>
<p align="center">
  <b>
    A <a href="https://www.nushell.sh">nushell</a> 
    <a href="https://www.nushell.sh/book/plugins.html">plugin</a> for syntax 
    highlighting.
  </b>
</p>

<br>

<p align="center">
  <a href="https://crates.io/crates/nu-plugin-highlight">
    <img alt="Version" src="https://img.shields.io/crates/v/nu-plugin-highlight?style=for-the-badge"/>
  </a>
  <a href="https://github.com/cptpiepmatz/nu-plugin-highlight/blob/main/LICENSE">
    <img alt="License" src="https://img.shields.io/crates/l/nu-plugin-highlight?style=for-the-badge"/>  
  </a>
</p>

## About
`nu-plugin-highlight` is a plugin for [Nushell](https://www.nushell.sh) that 
provides syntax highlighting for source code. 
It uses the [`syntect`](https://crates.io/crates/syntect) library for syntax 
highlighting and the [`bat`](https://crates.io/crates/bat) library for easy 
access to its ready-to-use assets.

## Usage
The `highlight` command can be used for syntax highlighting source code. 
Here are a few examples:
```nushell 
# Highlight a TOML file by its file extension
open Cargo.toml -r | highlight toml

# Highlight a Rust file by programming language name
open src/main.rs | highlight Rust

# Highlight a bash script by inferring the language (the file should start with a shebang)
open example.sh | highlight

# Highlight a TOML file with a different theme
open Cargo.toml -r | highlight toml -t ansi

# List all available themes
highlight --list-themes
```

### Parameters
- `language <string>`:
  This is an optional parameter that can be used to specify the language or file 
  extension to aid language detection.

### Flags
- `-h, --help`: 
  Display the help message for the highlight command.

- `-t, --theme <string>`: 
  The theme used for highlighting.

- `--list-themes`: 
  List all possible themes.

## Configuration
⚠️ Note: starting with Nushell **v0.83**, `let-env` is deprecated in favor of `$env.`, as informed in https://www.nushell.sh/blog/2023-07-25-nushell_0_83.html#breaking-changes

The plugin can be configured using environment variables:

### NU_PLUGIN_HIGHLIGHT_TRUE_COLORS
Enable or disable true colors (24-bit). 
By default, this is enabled.
```nushell
$env.NU_PLUGIN_HIGHLIGHT_TRUE_COLORS = true
```

### NU_PLUGIN_HIGHLIGHT_THEME
Set a theme to use.
The default theme depends on the operating system. 
Use `highlight --list-themes | where default == true` to see your default theme. 
Setting this environment variable should allow 
`highlight --list-themes | where id == $env.NU_PLUGIN_HIGHLIGHT_THEME` to result 
in a single row with your selected theme. 
If you get no results, you have set an invalid theme.
```nushell
$env.NU_PLUGIN_HIGHLIGHT_THEME = ansi
```

## Plugin Installation
Installing and registering the `nu-plugin-highlight` is a straightforward 
process. 
Follow these steps:

1. Install the plugin from crates.io using cargo:
    ```nushell
    cargo install nu_plugin_highlight
    ```

2. Restart your terminal session to ensure the newly installed plugin is recognized.

3. Find path of your installation:
    ```nushell
    which nu_plugin_highlight
    ```

4. Register the plugin with Nushell:
    ```nushell
    register path/to/the/plugin/binary
    ```

After registering, the plugin is available as part of your set of commands:

```nushell
help commands | where command_type == "plugin"
```

## License
`nu_plugin_highlight` is licensed under the MIT License. 
See [LICENSE](LICENSE) for more information.
