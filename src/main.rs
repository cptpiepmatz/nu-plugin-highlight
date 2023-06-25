use nu_plugin::{EvaluatedCall, LabeledError, MsgPackSerializer, Plugin, serve_plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Value};
use plugin::HighlightPlugin;

mod highlight;
mod plugin;

fn main() {
    serve_plugin(&mut HighlightPlugin::new(), MsgPackSerializer);
}
