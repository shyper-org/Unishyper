use core::ops::{Index, IndexMut};
use core::fmt::{Debug, Formatter, Result as FmtResult};

use gimli::Register;

#[macro_export]
macro_rules! registers {
    ($struct_name:ident, { $($name:ident = ($val:expr, $disp:expr)),+ $(,)? }) => {
        #[allow(missing_docs)]
        impl $struct_name {
            $(
                pub const $name: Register = Register($val);
            )+
        }

        impl $struct_name {
            /// The name of a register, or `None` if the register number is unknown.
            #[allow(dead_code)]
            pub fn register_name(register: Register) -> Option<&'static str> {
                match register {
                    $(
                        Self::$name => Some($disp),
                    )+
                    _ => return None,
                }
            }
        }
    };
}

#[derive(Clone)]
pub struct Registers {
    registers: [Option<u64>; 32],
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            registers: [None; 32],
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for (i, reg) in self.registers.iter().enumerate() {
            match *reg {
                None => {} // write!(fmt, "[{}]: None, ", i)?,
                Some(r) => write!(fmt, "[{}]: {:#X}, ", i, r)?,
            }
        }
        Ok(())
    }
}

impl Index<gimli::Register> for Registers {
    type Output = Option<u64>;
    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[index.0 as usize]
    }
}

impl IndexMut<gimli::Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.registers[index.0 as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Aarch64;

registers!(Aarch64, {
    X0 = (0, "X0"),
    X1 = (1, "X1"),
    X2 = (2, "X2"),
    X3 = (3, "X3"),
    X4 = (4, "X4"),
    X5 = (5, "X5"),
    X6 = (6, "X6"),
    X7 = (7, "X7"),
    X8 = (8, "X8"),
    X9 = (9, "X9"),
    X10 = (10, "X10"),
    X11 = (11, "X11"),
    X12 = (12, "X12"),
    X13 = (13, "X13"),
    X14 = (14, "X14"),
    X15 = (15, "X15"),
    X16 = (16, "X16"),
    X17 = (17, "X17"),
    X18 = (18, "X18"),
    X19 = (19, "X19"),
    X20 = (20, "X20"),
    X21 = (21, "X21"),
    X22 = (22, "X22"),
    X23 = (23, "X23"),
    X24 = (24, "X24"),
    X25 = (25, "X25"),
    X26 = (26, "X26"),
    X27 = (27, "X27"),
    X28 = (28, "X28"),
    X29 = (29, "X29"),
    X30 = (30, "X30"),
    SP = (31, "SP"),
});

pub const GPR_NUM_MAX: usize = 31;

#[cfg(feature = "unwind")]
pub const REG_RETURN_ADDRESS: Register = Aarch64::X30;
#[cfg(feature = "unwind")]
pub const REG_STACK_POINTER: Register = Aarch64::SP;
#[cfg(feature = "unwind")]
pub const REG_ARGUMENT: Register = Aarch64::X0;
