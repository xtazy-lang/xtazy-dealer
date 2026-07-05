mod app;
mod cli;
mod compiler_contract;
mod constants;
mod doctor;
mod error;
mod names;
mod package;
mod project;
mod rust_backend;
mod scaffold;
mod state;
mod support;
#[cfg(test)]
mod test_support;
mod toolchain;
mod update;
mod workflow;

fn main() {
    app::run();
}
