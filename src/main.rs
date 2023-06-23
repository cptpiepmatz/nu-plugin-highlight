use nu_plugin::{EvaluatedCall, LabeledError, MsgPackSerializer, Plugin, serve_plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Value};
use plugin::Highlight;

mod highlight;
mod plugin;

fn main() {
    serve_plugin(&mut Highlight::new(), MsgPackSerializer);
}
