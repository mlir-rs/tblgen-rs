// Demonstrates parsing an inline TableGen source string and listing all
// classes and definitions found in the record keeper.

use tblgen::{RecordKeeper, TableGenParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        class Animal<string name, int legs> {
            string Name = name;
            int Legs = legs;
        }

        class Pet<string name, int legs> : Animal<name, legs>;

        def Dog : Pet<"Dog", 4>;
        def Cat : Pet<"Cat", 4>;
        def Snake : Animal<"Snake", 0>;
        def Goldfish : Animal<"Goldfish", 0>;
    "#;

    let keeper: RecordKeeper = TableGenParser::new().add_source(source)?.parse()?;

    println!("=== Classes ===");
    for (name, class) in keeper.classes() {
        println!("  {}", name?);
        for field in class.values() {
            println!("    field: {}", field.name.to_str()?);
        }
    }

    println!("\n=== Defs ===");
    for (name, _def) in keeper.defs() {
        println!("  {}", name?);
    }

    println!("\n=== Pets (derived from Pet) ===");
    for def in keeper.all_derived_definitions("Pet") {
        let name = def.string_value("Name")?;
        let legs = def.int_value("Legs")?;
        println!("  {} has {} legs", name, legs);
    }

    Ok(())
}
