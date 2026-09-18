//! Regenerate the published JSON Schema.
//!
//! ```text
//! cargo run -p montaget-core --example schema > schema/montaget.schema.json
//! ```
//!
//! The committed copy and this generator's output are compared by a test, so a change to
//! the types that is not followed by a run of this command fails the suite rather than
//! shipping a schema that describes an older format.

fn main() {
    print!("{}", montaget_core::schema::generated_bytes());
}
