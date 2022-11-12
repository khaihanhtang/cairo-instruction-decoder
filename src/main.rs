use std::fmt::format;
use std::string::ToString;

fn extract_bits(instruction_u64: &u64, starting_index: usize, length: usize) -> u64 {
    (instruction_u64 >> starting_index) & ((1u64 << length) - 1)
}

#[derive(Debug)]
struct DecodedInstruction {
    off_dst: i16, // 16 bits
    off_op0: i16, // 16 bits
    off_op1: i16, // 16 bits
    dst_reg: u8, // 1 bit
    op0_reg: u8, // 1 bit
    op1_src: u8, // 3 bits
    res_logic: u8, // 2 bits
    pc_update: u8, // 3 bits
    ap_update: u8, // 2 bits
    opcode: u8, // 3 bits
}

impl DecodedInstruction {
    const UNDEFINED_BEHAVIOR: &str = "Undefined Behavior";
    const UNUSED: &str = "Unused";
    
    pub fn new(instruction_u64: &u64) -> Self {
        Self {
            off_dst: extract_bits(&instruction_u64, 0, 16).wrapping_sub(1 << 15) as i16,
            off_op0: extract_bits(&instruction_u64, 16, 16).wrapping_sub(1 << 15) as i16,
            off_op1: extract_bits(&instruction_u64, 32, 16).wrapping_sub(1 << 15) as i16,
            dst_reg: extract_bits(&instruction_u64, 48, 1) as u8,
            op0_reg: extract_bits(&instruction_u64, 49, 1) as u8,
            op1_src: extract_bits(&instruction_u64, 50, 3) as u8,
            res_logic: extract_bits(&instruction_u64, 53, 2) as u8,
            pc_update: extract_bits(&instruction_u64, 55, 3) as u8,
            ap_update: extract_bits(&instruction_u64, 58, 2) as u8,
            opcode: extract_bits(&instruction_u64, 60, 3) as u8,
        }
    }

    fn transform_number_to_computation_string(num: i16) -> String {
        if num < 0 {
            return format!(" - {}", -(num as i32));
        } else if num == 0 {
            return format!("");
        } else {
            return format!(" + {}", num);
        }
    }

    pub fn to_string(&self) -> String {
        // determine op0
        let mut op0_str: String;
        if self.op0_reg == 0 {
            op0_str = format!("[ap{}]",
                              Self::transform_number_to_computation_string(self.off_op0)
            );
        } else {
            op0_str = format!("[fp{}]",
                              Self::transform_number_to_computation_string(self.off_op0)
            );
        }

        // determine op1 and instruction_size
        let mut instruction_size: usize;
        let mut op1_str: String;
        match self.op1_src {
            0 => {
                instruction_size = 1;
                op1_str = format!("[{}{}]",
                                  op0_str,
                                  Self::transform_number_to_computation_string(self.off_op1)
                );
            },
            1 => {
                instruction_size = 2;
                op1_str = format!("[pc{}]",
                                  Self::transform_number_to_computation_string(self.off_op1)
                );
            },
            2 => {
                instruction_size = 1;
                op1_str = format!("[fp{}]",
                                  Self::transform_number_to_computation_string(self.off_op1)
                );
            },
            4 => {
                instruction_size = 1;
                op1_str = format!("[ap{}]",
                                  Self::transform_number_to_computation_string(self.off_op1)
                );
            },
            _ => {
                return Self::UNDEFINED_BEHAVIOR.to_string();
            }
        }

        // determine res
        let mut res_str: String;
        if self.pc_update == 4 {
            if self.res_logic == 0 && self.opcode == 0 && self.ap_update != 1 {
                res_str = Self::UNUSED.to_string();
            } else {
                return Self::UNDEFINED_BEHAVIOR.to_string();
            }
        } else if self.pc_update == 0 || self.pc_update == 1 || self.pc_update == 2 {
            match self.res_logic {
                0 => {
                    res_str = op1_str.clone();
                },
                1 => {
                    res_str = format!("{} + {}", op0_str, op1_str.clone());
                },
                2 => {
                    res_str = format!("{} * {}", op0_str, op1_str.clone());
                },
                _ => {
                    return Self::UNDEFINED_BEHAVIOR.to_string();
                }
            }
        } else {
            return Self::UNDEFINED_BEHAVIOR.to_string();
        }

        // determine dst
        let mut dst_str: String;
        if self.dst_reg == 0 {
            dst_str = format!("[ap{}]",
                              Self::transform_number_to_computation_string(self.off_dst)
            );
        } else {
            dst_str = format!("[fp{}]",
                              Self::transform_number_to_computation_string(self.off_dst)
            );
        }

        // determine next_pc
        let mut pc_str: String;
        match self.pc_update {
            0 => { // common case
                pc_str = format!("jump comm pc += {}", instruction_size);
            },
            1 => { // absolute jump
                pc_str = format!("jump abs pc = {}", res_str);
            },
            2 => { // relative jump
                pc_str = format!("jump rel pc += {}", res_str);
            },
            4 => { // conditional relative jump
                pc_str = format!("jump rel cond pc += (1 - {}) * {} + {} * {}",
                    dst_str, instruction_size,
                    dst_str, op1_str.clone(),
                );
            },
            _ => {
                return Self::UNDEFINED_BEHAVIOR.to_string();
            }
        }

        // determine new value of ap and fp based on opcode
        let mut opcode_str: String;
        let mut fp_str = format!("");
        let mut ap_str = format!("");
        if self.opcode == 1 {
            opcode_str = format!("call");
            fp_str = format!("fp = ap + 2");
            match self.ap_update {
                0 => {
                    ap_str = format!("ap += 2");
                },
                _ => {
                    return Self::UNDEFINED_BEHAVIOR.to_string();
                }
            }
        } else if self.opcode == 0 || self.opcode == 2 || self.opcode == 4 {
            match self.ap_update {
                0 => {
                    ap_str = format!("");
                },
                1 => {
                    ap_str = format!("ap += {}", res_str);
                },
                2 => {
                    ap_str = format!("ap += 1");
                },
                _ => {
                    return Self::UNDEFINED_BEHAVIOR.to_string();
                }
            }

            match self.opcode {
                0 => {
                    opcode_str = format!("");
                },
                2 => {
                    opcode_str = format!("ret");
                    fp_str = format!("fp = {}", dst_str);
                }
                4 => {
                    opcode_str = format!("assert equal {} = {}", res_str, dst_str);
                }
                _ => {
                    return Self::UNDEFINED_BEHAVIOR.to_string();
                }
            }
        } else {
            return Self::UNDEFINED_BEHAVIOR.to_string();
        }

        let mut returned_string = format!("");
        if !opcode_str.is_empty() {
            returned_string = format!("{}\n{};", returned_string, opcode_str);
        }

        if !pc_str.is_empty() {
            returned_string = format!("{}\n{};", returned_string, pc_str);
        }

        if !fp_str.is_empty() {
            returned_string = format!("{}\n{};", returned_string, fp_str);
        }

        if !ap_str.is_empty() {
            returned_string = format!("{}\n{};", returned_string, ap_str);
        }

        returned_string
    }
}

fn main() {
    let instruction = 0x48307ffe7fff8000u64;
    let decoded_instruction = DecodedInstruction::new(&instruction);
    println!("{:?}", decoded_instruction);
    println!("{}", decoded_instruction.to_string())
}
