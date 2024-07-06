//! A cpu for the NES 6502 instruction set.
//!
//! This module contains the [`CPU`] type
//!
//! ---
//!
//! <div align="left">
//!
//! #### NES CPU Memory Map
//!
//! |  | Start | End |  
//! | ---:  | :---: | :---: |
//! | **CPU RAM** | `0x0000` | `0x2000` |
//! | **IO Registers** | `0x2000` | `0x4020` |
//! | **Expansion Rom** | `0x4020` | `0x6000` |
//! | **Save RAM** | `0x6000` | `0x8000` |
//! | **Program ROM** | `0x8000` | `0xFFFF` |
//!
//! </div>
//!
//!
//! [CPU RAM] can be accessed from addresses: `[ 0x0000 => 0x2000 ]`
//!
//! NES hardware access [IO Registers] for PPU, APU, and GamePads: `[ 0x2000 => 0x4020 ]`
//!
//! [Expansion Rom] Used differently by various cartridge generations, controlled by mappers: `[ 0x4020 => 0x6000 ]`
//!
//! [Save RAM] reserved for cartridge RAM if available, used for saving game states:  `[ 0x6000 => 0x8000 ]`
//!
//! [Program Rom] space on a cartridge. `[ 0x8000 to 0x10000 ]`
//!
//!
//! NES CPU has 6 Registers:

//! - Program Counter (PC) - stores the address for the next instruction.

//! - Stack Pointer - Memory space [0x0100 .. 0x1FF] is used for stack. The stack pointer holds the address of the top of that space. NES Stack (as all stacks) grows from top to bottom: when a byte gets pushed to the stack, SP register decrements. When a byte is retrieved from the stack, SP register increments.
//!
//! - Accumulator (A) - stores the results of arithmetic, logic, and memory access operations. It used as an input parameter for some operations.
//!
//! - Index Register X (X) - used as an offset in specific memory addressing modes (more on this later). Can be used for auxiliary storage needs (holding temp values, being used as a counter, etc.)
//!
//! - Index Register Y (Y) - similar use cases as register X.
//!
//! - Processor status (P) - 8-bit register represents 7 status flags that can be set or unset depending on the result of the last executed instruction (for example Z flag is set (1) if the result of an operation is 0, and is unset/erased (0) otherwise)
//!

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            register_a: 0u8,
            register_x: 0u8,
            status: 0u8,
            program_counter: 0u16,
            memory: [0u8; 0xFFFF],
        }
    }
}
impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            memory: [0u8; 0xFFFF],
        }
    }

    /// Returns data stored within CPU memory
    /// * `addr` - An u16 sized address that corresponds to an address in memory
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    /// ## LDA - Load Accumulator
    /// Loads a byte of memory into the accumulator setting the zero and negative flags as appropriate.
    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    /// ## INX - Increment X Register
    /// Adds one to the X register setting the zero and negative flags as appropriate.
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x)
    }

    /// Helper function that manipulates CPU status on zero and negative flags
    fn update_zero_and_negative_flags(&mut self, register: u8) {
        self.update_zero_flag(register);
        self.update_negative_flag(register);
    }

    /// Negative Flag is set if bit 7 is set: 0x1000_0000 & accumulator
    fn update_negative_flag(&mut self, register: u8) {
        match register & 0b1000_0000 {
            0 => self.status &= 0b0111_1111, // if no bit, turn off negative bit in status
            _ => self.status |= 0b1000_0000, // if bit, turn on negative bit in status
        }
    }

    /// Zero Flag is set if accumulator = 0
    fn update_zero_flag(&mut self, register: u8) {
        match register {
            0 => self.status |= 0b0000_0010,  // zero, turn on zero bit in status
            _ => self.status &= &0b1111_1101, // not zero, turn off zero bit in status
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.lda(param);
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::default();

        // Assign value 0x05 to register_a, break
        let program = vec![0xa9, 0x05, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.register_a, 0x05); // Register A should hold 0x05
        assert_eq!(cpu.status, 0); // Status should not change
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();

        // Assign zero to accumulator, break
        let program = vec![0xa9, 0x00, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.status & 0b0000_0010, 0b10); // Ensure zero flag is set
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();

        // Assign negative to accumulator, break
        let program = vec![0xa9, 0x80, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.status & 0x80, 0x80); // Ensure negative flag is set
    }
    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();

        // Move 0xff into register_a, copy register_a to register_x, break
        let program = vec![0xa9, 0xff, 0xaa, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.register_x, 0xFF); // register_x should hold register_a value
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();

        // Move 0xc0 into register_a, copy register_a to register_x, increment register_x, break;
        let program = vec![0xa9, 126, 0xaa, 0xe8, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.register_x, 127); // register_x should hold register_a value + 1
        assert_eq!(cpu.status, 0);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();

        // add 1 to register x, add 1 to register x, break
        let program = vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00];
        cpu.load_and_run(program);
        assert_eq!(cpu.register_x, 1);
    }
}
