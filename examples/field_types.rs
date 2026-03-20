// Demonstrates reading the various field types supported by TableGen:
// bit, bits, int, string, list, dag, and def references.

use tblgen::{TableGenParser, init::TypedInit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        class Reg<string n> { string Name = n; }
        def RegA : Reg<"rA">;
        def RegB : Reg<"rB">;
        def ADD;

        def MyRecord {
            bit       flag   = 1;
            bits<8>   opcode = { 0, 0, 1, 0, 1, 1, 0, 1 };
            int       size   = 64;
            string    name   = "my_record";
            list<int> values = [10, 20, 30];
            dag       instr  = (ADD RegA:$dst, RegB:$src);
            Reg       reg    = RegA;
        }
    "#;

    let keeper = TableGenParser::new().add_source(source)?.parse()?;
    let rec = keeper.def("MyRecord")?;

    // bit
    let flag: bool = rec.bit_value("flag")?;
    println!("flag       = {}", flag);

    // bits<8> — try_into returns Err if any bit is a variable reference
    let opcode: Vec<bool> = rec.bits_value("opcode")?;
    println!("opcode     = {:?}", opcode);

    // int
    let size: i64 = rec.int_value("size")?;
    println!("size       = {}", size);

    // string
    let name: String = rec.string_value("name")?;
    println!("name       = {:?}", name);

    // list<int>
    let list = rec.list_init_value("values")?;
    print!("values     = [");
    for (i, elem) in list.iter().enumerate() {
        let n: i64 = elem.try_into()?;
        if i > 0 {
            print!(", ");
        }
        print!("{}", n);
    }
    println!("]");

    // dag
    let dag = rec.dag_value("instr")?;
    println!("instr op   = {}", dag.operator().name()?);
    for (arg_name, init) in dag.args() {
        match arg_name {
            Some(n) => println!("  arg ${} = {}", n, init),
            None => println!("  arg     = {}", init),
        }
    }

    // def reference
    let def_ref = rec.def_value("reg")?;
    println!("reg        = {}", def_ref.name()?);

    // Iterating all fields dynamically
    println!("\n--- all fields ---");
    for field in rec.values() {
        let type_name = match field.init {
            TypedInit::Bit(_) => "bit",
            TypedInit::Bits(_) => "bits",
            TypedInit::Int(_) => "int",
            TypedInit::String(_) => "string",
            TypedInit::List(_) => "list",
            TypedInit::Dag(_) => "dag",
            TypedInit::Def(_) => "def",
            TypedInit::Code(_) => "code",
            TypedInit::Invalid => "invalid",
        };
        println!("  {:10} : {}", field.name.to_str()?, type_name);
    }

    Ok(())
}
