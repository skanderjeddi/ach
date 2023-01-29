use core::fmt;
use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ACHColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ACHColorCode(u8);

impl ACHColorCode {
    fn new(foreground: ACHColor, background: ACHColor) -> ACHColorCode {
        ACHColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ACHCharacter {
    ascii: u8,
    color: ACHColorCode,
}

#[repr(transparent)]
struct ACHBuffer {
    chars: [[Volatile<ACHCharacter>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct ACHWriter {
    column: usize, 
    color: ACHColorCode,
    buffer: &'static mut ACHBuffer,
}

impl ACHWriter {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                self.buffer.chars[row][self.column].write(ACHCharacter {
                    ascii: byte,
                    color: self.color,
                });
                self.column += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                0x20..=0x7E | b'\n' => self.write_byte(b),
                _ => self.write_byte(0xFE),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let b = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(b);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ACHCharacter {
            ascii: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for ACHWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<ACHWriter> = Mutex::new(ACHWriter {
        column: 0,
        color: ACHColorCode::new(ACHColor::Yellow, ACHColor::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut ACHBuffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}