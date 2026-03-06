// Demonstrates error handling with source location information.
// When a field access fails, the error carries a source location that can be
// enriched with the original TableGen source via `add_source_info`.

use tblgen::TableGenParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        def MyRecord {
            int value = 42;
        }
    "#;

    let keeper = TableGenParser::new().add_source(source)?.parse()?;
    let rec = keeper.def("MyRecord")?;

    // Correct access: works fine.
    println!("value = {}", rec.int_value("value")?);

    // Wrong type: requesting a string from an int field returns an error.
    if let Err(e) = rec.string_value("value") {
        // Without source info: prints the error message only.
        println!("\nWithout source info:\n  {}", e);

        // With source info: LLVM's SourceMgr prints the offending line with a caret.
        println!("\nWith source info:");
        println!("{}", e.add_source_info(keeper.source_info()));
    }

    // Missing field: accessing a field that does not exist.
    if let Err(e) = rec.int_value("nonexistent") {
        println!("Missing field error: {}", e);
    }

    // Missing def: requesting a def that was not defined.
    if let Err(e) = keeper.def("DoesNotExist") {
        println!("Missing def error:   {}", e);
    }

    Ok(())
}
