#![allow(non_snake_case)]

use toyBitcoinScript::{Opcode, ToyScript, hash160};

fn main() -> Result<(), &'static str> {
    println!("Testing Bitcoin ToyScript Engine\n");

    println!("Running: OP_2 OP_3 OP_ADD");
    let script = ToyScript::new(vec![
        Opcode::Op2 as u8,
        Opcode::Op3 as u8,
        Opcode::OpAdd as u8,
    ]);
    let stack = script.run()?;
    println!("Stack: {:?}\n", stack);

    println!("Running: <preimage> OP_HASH160 <hash> OP_EQUALVERIFY");
    let secret = b"secret-passphrase";
    let target_hash = hash160(secret);

    let mut bytecode = Vec::new();
    bytecode.push(secret.len() as u8);
    bytecode.extend_from_slice(secret);
    bytecode.push(Opcode::OpHash160 as u8);
    bytecode.push(target_hash.len() as u8);
    bytecode.extend_from_slice(&target_hash);
    bytecode.push(Opcode::OpEqualVerify as u8);

    let final_stack = ToyScript::new(bytecode).run()?;
    println!(
        "Hash verification succeeded! Stack empty: {}\n",
        final_stack.is_empty()
    );

    println!("Running the same script with the wrong preimage");
    let mut bad = Vec::new();
    bad.push(5);
    bad.extend_from_slice(b"wrong");
    bad.push(Opcode::OpHash160 as u8);
    bad.push(target_hash.len() as u8);
    bad.extend_from_slice(&target_hash);
    bad.push(Opcode::OpEqualVerify as u8);

    match ToyScript::new(bad).run() {
        Ok(_) => println!("BUG: wrong preimage was accepted"),
        Err(e) => println!("Correctly rejected: {}", e),
    }

    Ok(())
}
