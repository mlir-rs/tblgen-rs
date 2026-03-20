// Demonstrates reading the various field types supported by TableGen:
// bit, bits, int, string, list, dag, def references, and the new
// typed value accessors and metadata APIs.

use tblgen::{TableGenParser, init::TypedInit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        class Reg<string n> { string Name = n; }
        def RegA : Reg<"rA">;
        def RegB : Reg<"rB">;
        def ADD;

        class Base;

        def MyRecord : Base {
            bit          flag   = 1;
            bits<8>      opcode = { 0, 0, 1, 0, 1, 1, 0, 1 };
            int          size   = 64;
            string       name   = "my_record";
            list<int>    values = [10, 20, 30];
            list<string> tags   = ["fast", "alu"];
            list<Reg>    regs   = [RegA, RegB];
            dag          instr  = (ADD RegA:$dst, RegB:$src);
            Reg          reg    = RegA;
            string       label  = ?;
        }
    "#;

    let keeper = TableGenParser::new().add_source(source)?.parse()?;
    let rec = keeper.def("MyRecord")?;

    // bit
    let flag: bool = rec.bit_value("flag")?;
    println!("flag       = {}", flag);

    // bits<8>
    let opcode: Vec<bool> = rec.bits_value("opcode")?;
    println!("opcode     = {:?}", opcode);

    // int
    let size: i64 = rec.int_value("size")?;
    println!("size       = {}", size);

    // string (as &str, zero-copy)
    let name: &str = rec.str_value("name")?;
    println!("name       = {:?}", name);

    // list<int> via list_of_ints_value (direct, no per-element casting)
    let values = rec.list_of_ints_value("values")?;
    println!("values     = {:?}", values);

    // list<string> via list_of_strings_value
    let tags = rec.list_of_strings_value("tags")?;
    println!("tags       = {:?}", tags);

    // list<Reg> via list_of_defs_value
    let regs = rec.list_of_defs_value("regs")?;
    print!("regs       = [");
    for (i, r) in regs.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}", r.name()?);
    }
    println!("]");

    // list<int> via list_init_value (element-by-element iteration)
    let list = rec.list_init_value("values")?;
    print!("values (iter) = [");
    for (i, elem) in list.iter().enumerate() {
        let n: i64 = elem.try_into()?;
        if i > 0 {
            print!(", ");
        }
        print!("{}", n);
    }
    println!("]");

    // dag with arg_no lookup
    let dag = rec.dag_value("instr")?;
    println!("instr op   = {}", dag.operator().name()?);
    if let Some(idx) = dag.arg_no("dst") {
        println!("  $dst at index {}", idx);
    }
    for (arg_name, init) in dag.args() {
        match arg_name {
            Some(n) => println!("  arg ${} = {}", n, init),
            None => println!("  arg     = {}", init),
        }
    }

    // def reference
    let def_ref = rec.def_value("reg")?;
    println!("reg        = {}", def_ref.name()?);

    // optional string (unset field)
    let label = rec.optional_str_value("label")?;
    println!("label      = {:?}", label); // None

    // is_value_unset
    println!("label unset? {}", rec.is_value_unset("label"));

    // Record metadata
    println!("\n--- record metadata ---");
    println!("is_class     = {}", rec.is_class());
    println!("anonymous    = {}", rec.anonymous());
    println!("id           = {}", rec.id());
    println!("name_init    = {}", rec.name_init());
    println!("def_init     = {}", rec.def_init());

    // Superclass checks
    let base = keeper.class("Base")?;
    println!("has_direct_super_class(Base) = {}", rec.has_direct_super_class(base));
    println!("type_is_subclass_of(Base)    = {}", rec.type_is_subclass_of(base));
    println!("num_type_classes             = {}", rec.num_type_classes());

    // RecordValue metadata
    println!("\n--- field metadata ---");
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
        let extra = if let Some(w) = field.bits_width() {
            format!("<{}>", w)
        } else if let Some(_) = field.list_element_type() {
            format!("<...>")
        } else {
            String::new()
        };
        println!(
            "  {:10} : {}{:6}  template_arg={}  nonconcrete_ok={}",
            field.name.to_str()?,
            type_name,
            extra,
            field.is_template_arg(),
            field.is_nonconcrete_ok(),
        );
    }

    Ok(())
}
