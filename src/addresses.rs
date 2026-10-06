pub struct ToyScript{
    byte: Vec<u8>,
}

pub enum Action<'a> {
    ExecuteOpcode(Opcode),
    PushData(&'a [u8]),
}

impl ToyScript {
    pub fn new(byte: Vec<u8>) -> Self{
        Self { byte }
     }

    pub fn read_action(&self, cursor: usize) -> Result< (Action<'_>, usize) , &'static str > {
        if cursor >= self.byte.len() {
            return Err("you have exceeded the available bytes");
        }

        let op = self.byte[cursor];
        match op{
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
            Ok((Action::ExecuteOpcode(op), 1))
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
            // push constants
            Opcode::Op1Negate => stack.push(vec![0x81]),
            Opcode::Op1 => stack.push(vec![1]),
            Opcode::Op2 => stack.push(vec![2]),
            Opcode::Op3 => stack.push(vec![3]),

            // flow control
            Opcode::OpVerify => {
                if !is_true(&stack.pop()?) {
                    return Err("OpVerify failed");
                }
            }
            Opcode::OpReturn => return Err("OpReturn: script terminated"),

            // stack manipulation
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
                let a = stack.pop()?; // top
                let b = stack.pop()?;
                stack.push(a);
                stack.push(b);
            }

            // comparison
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

            // arithmetic
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

            // crypto
            Opcode::OpHash160 => {
                let data = stack.pop()?;
                stack.push(hash160(&data));
            }
            Opcode::OpCheckSig => return Err("OpCheckSig not implemented"),
        },
    }
    Ok(())

    fn is_true(b: &[u8]) -> bool {
    b.iter().any(|&x| x != 0)
}

fn decode_num(b: &[u8]) -> Result<u64, &'static str> {
    if b.len() > 4 { return Err("script number too long"); }
    Ok(b.iter().enumerate().fold(0, |n, (i, &x)| n | (x as u64) << (8 * i)))
}

fn encode_num(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    while n > 0 {
        out.push(n as u8);
        n >>= 8;
    }
    out 
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
  
struct Stack{
    stack: Vec<Vec<u8>>
}

impl Stack{

    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, item: Vec<u8>) {
        self.stack.push(item)
    }
 
    pub fn pop(&mut self) -> Result<Vec<u8>, &'static str> {
        self.stack.pop().ok_or("there is an overflow of the stack")
    }

    pub fn top(&self) -> Result<&[u8], &'static str> {
        self.stack.last().map(|v| v.as_slice()).ok_or("there is an overflow of the stack")
    }

    pub fn last_item(&self) -> bool{
        self.stack.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Op1Negate = 0x4f,
    Op1       = 0x51,
    Op2       = 0x52,
    Op3       = 0x53,
    OpVerify  = 0x69,
    OpReturn  = 0x6a,
    OpToAltStack   = 0x6b,
    OpFromAltStack = 0x6c,
    OpDrop         = 0x75,
    OpDup          = 0x76,
    OpSwap         = 0x78,
    OpEqual       = 0x87,
    OpEqualVerify = 0x88,
    Op1Add = 0x8b,
    Op1Sub = 0x8c,
    OpAdd  = 0x93,
    OpSub  = 0x94,
    OpHash160  = 0xa9,
    OpCheckSig = 0xac,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x4f => Ok(Opcode::Op1Negate),
            0x51 => Ok(Opcode::Op1),
            0x52 => Ok(Opcode::Op2),
            0x53 => Ok(Opcode::Op3),

            0x69 => Ok(Opcode::OpVerify),
            0x6a => Ok(Opcode::OpReturn),

            0x6b => Ok(Opcode::OpToAltStack),
            0x6c => Ok(Opcode::OpFromAltStack),
            0x75 => Ok(Opcode::OpDrop),
            0x76 => Ok(Opcode::OpDup),
            0x78 => Ok(Opcode::OpSwap),

            0x87 => Ok(Opcode::OpEqual),
            0x88 => Ok(Opcode::OpEqualVerify),

            0x8b => Ok(Opcode::Op1Add),
            0x8c => Ok(Opcode::Op1Sub),
            0x93 => Ok(Opcode::OpAdd),
            0x94 => Ok(Opcode::OpSub),

            0xa9 => Ok(Opcode::OpHash160),
            0xac => Ok(Opcode::OpCheckSig),

            _ => Err("Unknown or unsupported opcode"),
        }
    }
}