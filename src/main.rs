use nu_plugin::{serve_plugin, MsgPackSerializer};
use plugin::HighlightPlugin;

mod highlight;
mod plugin;
mod terminal;
mod theme;

/// The main function that serves the plugin using MsgPackSerializer.
fn main() {
    serve_plugin(&mut HighlightPlugin::new(), MsgPackSerializer);
}
