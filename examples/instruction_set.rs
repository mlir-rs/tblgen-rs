// A more realistic example modelling a simple instruction set architecture.
// Shows how tblgen-rs can be used to drive code generation from TableGen
// descriptions, which is the primary use case of this crate.

use tblgen::{RecordKeeper, TableGenParser};

const SOURCE: &str = r#"
    // Register file
    class Register<string name, int index> {
        string Name  = name;
        int    Index = index;
    }

    def R0 : Register<"r0", 0>;
    def R1 : Register<"r1", 1>;
    def R2 : Register<"r2", 2>;
    def R3 : Register<"r3", 3>;

    // Instruction formats
    class Instruction<string mnemonic, bits<6> opcode, int operands> {
        string   Mnemonic = mnemonic;
        bits<6>  Opcode   = opcode;
        int      Operands = operands;
    }

    class ALUInstr<string mnemonic, bits<6> opcode>
        : Instruction<mnemonic, opcode, 3>;

    class MemInstr<string mnemonic, bits<6> opcode>
        : Instruction<mnemonic, opcode, 2>;

    def ADD  : ALUInstr<"add",  { 0, 0, 0, 0, 0, 1 }>;
    def SUB  : ALUInstr<"sub",  { 0, 0, 0, 0, 1, 0 }>;
    def AND  : ALUInstr<"and",  { 0, 0, 0, 0, 1, 1 }>;
    def OR   : ALUInstr<"or",   { 0, 0, 0, 1, 0, 0 }>;
    def LOAD : MemInstr<"load", { 1, 0, 0, 0, 0, 0 }>;
    def STR  : MemInstr<"str",  { 1, 0, 0, 0, 0, 1 }>;
"#;

fn print_registers(keeper: &RecordKeeper) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Registers ===");
    for reg in keeper.all_derived_definitions("Register") {
        let name = reg.string_value("Name")?;
        let index = reg.int_value("Index")?;
        println!("  {:4}  index={}", name, index);
    }
    Ok(())
}

fn print_instructions(keeper: &RecordKeeper) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Instructions ===");
    for instr in keeper.all_derived_definitions("Instruction") {
        let mnemonic = instr.string_value("Mnemonic")?;
        let operands = instr.int_value("Operands")?;
        let opcode: Vec<bool> = instr.bits_value("Opcode")?;

        // Render opcode bits as a binary string (MSB first in TableGen order)
        let bits: String = opcode.iter().map(|&b| if b { '1' } else { '0' }).collect();

        let kind = if instr.subclass_of("ALUInstr") {
            "ALU"
        } else if instr.subclass_of("MemInstr") {
            "MEM"
        } else {
            "???"
        };

        println!(
            "  {:6}  opcode={}  operands={}  kind={}",
            mnemonic, bits, operands, kind
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keeper = TableGenParser::new().add_source(SOURCE)?.parse()?;
    print_registers(&keeper)?;
    print_instructions(&keeper)?;
    Ok(())
}
