#![allow(non_snake_case)]

pub mod blockchain;
pub mod executionStack;
pub mod opcode;
pub mod p2p;
pub mod rpc;
pub mod transaction;

pub use executionStack::Stack;
pub use opcode::Opcode;

use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub struct ToyScript {
    byte: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<'a> {
    ExecuteOpcode(Opcode),
    PushData(&'a [u8]),
}

impl ToyScript {
    pub fn new(byte: Vec<u8>) -> Self {
        Self { byte }
    }

    pub fn read_action(&self, cursor: usize) -> Result<(Action<'_>, usize), &'static str> {
        if cursor >= self.byte.len() {
            return Err("you have exceeded the available bytes");
        }

        let op = self.byte[cursor];
        match op {
            0x00 => Ok((Action::PushData(&[]), 1)),
            0x01..=0x4b => {
                let len = op as usize;
                let start = cursor + 1;
                let end = start + len;

                if end > self.byte.len() {
                    return Err("Pushdata exceeds available bytes");
                }

                let data = &self.byte[start..end];
                Ok((Action::PushData(data), 1 + len))
            }
            _ => {
                let opcode = Opcode::try_from(op)?;
                Ok((Action::ExecuteOpcode(opcode), 1))
            }
        }
    }

    pub fn execute_action(
        &self,
        action: Action<'_>,
        stack: &mut Stack,
        alt: &mut Stack,
    ) -> Result<(), &'static str> {
        match action {
            Action::PushData(data) => stack.push(data.to_vec()),

            Action::ExecuteOpcode(op) => match op {
                Opcode::Op0 => stack.push(vec![]),
                Opcode::Op1Negate => stack.push(vec![0x81]),
                Opcode::Op1 => stack.push(vec![1]),
                Opcode::Op2 => stack.push(vec![2]),
                Opcode::Op3 => stack.push(vec![3]),
                Opcode::OpVerify => {
                    if !is_true(&stack.pop()?) {
                        return Err("OpVerify failed");
                    }
                }
                Opcode::OpReturn => return Err("OpReturn: script terminated"),
                Opcode::OpToAltStack => alt.push(stack.pop()?),
                Opcode::OpFromAltStack => stack.push(alt.pop()?),
                Opcode::OpDrop => {
                    stack.pop()?;
                }
                Opcode::OpDup => {
                    let top = stack.top()?.to_vec();
                    stack.push(top);
                }
                Opcode::OpSwap => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(a);
                    stack.push(b);
                }
                Opcode::OpEqual => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    stack.push(if a == b { vec![1] } else { vec![] });
                }
                Opcode::OpEqualVerify => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    if a != b {
                        return Err("OpEqualVerify failed");
                    }
                }
                Opcode::Op1Add => {
                    let n = decode_num(&stack.pop()?)?;
                    stack.push(encode_num(n + 1));
                }
                Opcode::Op1Sub => {
                    let n = decode_num(&stack.pop()?)?;
                    stack.push(encode_num(n - 1));
                }
                Opcode::OpAdd => {
                    let b = decode_num(&stack.pop()?)?;
                    let a = decode_num(&stack.pop()?)?;
                    stack.push(encode_num(a + b));
                }
                Opcode::OpSub => {
                    let b = decode_num(&stack.pop()?)?;
                    let a = decode_num(&stack.pop()?)?;
                    stack.push(encode_num(a - b));
                }
                Opcode::OpHash160 => {
                    let data = stack.pop()?;
                    stack.push(hash160(&data));
                }
                Opcode::OpCheckSig => return Err("OpCheckSig not implemented"),
            },
        }
        Ok(())
    }

    pub fn run(&self) -> Result<Stack, &'static str> {
        let mut stack = Stack::new();
        let mut alt = Stack::new();
        let mut cursor = 0;
        while cursor < self.byte.len() {
            let (action, consumed) = self.read_action(cursor)?;
            self.execute_action(action, &mut stack, &mut alt)?;
            cursor += consumed;
        }
        Ok(stack)
    }
}

pub fn is_true(b: &[u8]) -> bool {
    if b.is_empty() {
        return false;
    }
    for (i, &x) in b.iter().enumerate() {
        if x != 0 {
            if i == b.len() - 1 && x == 0x80 {
                return false;
            }
            return true;
        }
    }
    false
}

pub fn decode_num(b: &[u8]) -> Result<i64, &'static str> {
    if b.is_empty() {
        return Ok(0);
    }
    if b.len() > 4 {
        return Err("script number too long");
    }

    let mut result: i64 = 0;
    for (i, &byte) in b.iter().enumerate() {
        result |= (byte as i64) << (8 * i);
    }

    let last = *b.last().unwrap();
    if last & 0x80 != 0 {
        result &= !(0x80i64 << (8 * (b.len() - 1)));
        result = -result;
    }

    Ok(result)
}

pub fn encode_num(n: i64) -> Vec<u8> {
    if n == 0 {
        return Vec::new();
    }

    let is_negative = n < 0;
    let mut abs_val = if is_negative { -n as u64 } else { n as u64 };
    let mut out = Vec::new();

    while abs_val > 0 {
        out.push((abs_val & 0xff) as u8);
        abs_val >>= 8;
    }

    if let Some(last) = out.last_mut() {
        if *last & 0x80 != 0 {
            out.push(if is_negative { 0x80 } else { 0x00 });
        } else if is_negative {
            *last |= 0x80;
        }
    }

    out
}

pub fn hash160(data: &[u8]) -> Vec<u8> {
    let sha_digest = Sha256::digest(data);
    let ripemd_digest = Ripemd160::digest(sha_digest);
    ripemd_digest.to_vec()
}