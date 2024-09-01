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
Custom themes can be loaded too.

## Usage
The `highlight` command can be used for syntax highlighting source code. 
Here are a few examples:
```nushell
# Highlight a Markdown file by guessing the type from the pipeline metadata
open README.md | highlight

# Highlight a TOML file by its file extension
open Cargo.toml -r | echo $in | highlight toml

# Highlight a Rust file by programming language name
open src/main.rs | echo $in | highlight Rust

# Highlight a bash script by inferring the language (the file should start with a shebang)
open example.sh | echo $in | highlight

# Highlight a TOML file with a different theme
open Cargo.toml -r | highlight -t ansi

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
The plugin can be configured using the 
[`$env.config.plugins.highlight`](https://github.com/nushell/nushell/pull/10955) 
variable.

### `true_colors`
Enable or disable true colors (24-bit).
By default, this is enabled.
```nushell
$env.config.plugins.highlight.true_colors = true
```

### `theme`
Set a theme to use.
The default theme depends on the operating system.
Use `highlight --list-themes | where default == true` to see your default theme.
Setting this environment variable should allow
`highlight --list-themes | where id == $env.config.plugins.highlight.theme` to 
result in a single row with your selected theme.
If you get no results, you have set an invalid theme.
```nushell
$env.config.plugins.highlight.theme = ansi
```

### `custom_themes`
Set a directory to load custom themes from.
Using `synctect`s theme loader, you can load custom themes in the `.tmtheme` 
format from a directory that is passed as this configuration value.
```nushell
$env.config.plugins.highlight.custom_themes = ~/.nu/highlight/themes
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
   
    If you are using a version **lower** than **0.93.0**, use `register` instead of `plugin add`.
    ```nushell
    plugin add path/to/the/plugin/binary
    ```

5. Make the plugin available for use:

   Tip: You can simply restart the shell or terminal. When nushell starts, it loads all plugins.

   If you are using a version **lower** than **0.93.0**, you do **not need** to do this.
   ```nushell
   plugin use highlight
   ```

After registering, the plugin is available as part of your set of commands:

```nushell
help commands | where command_type == "plugin"
```

## Version Numbering
Starting with version `v1.1.0`, the version number of `nu-plugin-highlight` 
incorporates the version number of its dependency, `nu-plugin`. 
This is denoted in the format `v1.1.0+0.90.1`, where `v1.1.0` refers to the 
version of `nu-plugin-highlight` and `0.90.1` refers to the version of the 
`nu-plugin` dependency.

## License
`nu_plugin_highlight` is licensed under the MIT License. 
See [LICENSE](LICENSE) for more information.
