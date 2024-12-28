//! Let cargo know that we need to recompile when the migrations change.
fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
}
