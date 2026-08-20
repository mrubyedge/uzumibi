# What is mruby/edge?

[mruby/edge](https://github.com/mrubyedge/mrubyedge) is the Ruby runtime used by Uzumibi. It implements an mruby-compatible VM in Rust and is designed to run in WebAssembly environments.

In an Uzumibi build:

1. the project build script compiles `lib/app.rb` or `lib/consumer.rb` to mruby bytecode;
2. that bytecode is embedded in the application binary;
3. the platform adapter initializes a mruby/edge VM and evaluates the embedded bytecode;
4. requests or messages are dispatched to the Ruby application.

The available Ruby language and standard-library features are determined by mruby/edge and by the crates initialized by the selected template. Do not assume that every feature or native extension available in CRuby is present.

For runtime implementation details and compatibility information, refer to the [mruby/edge repository](https://github.com/mrubyedge/mrubyedge).
