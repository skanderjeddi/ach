#![no_std]
#![no_main]

mod vga;

use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum QEMUExitCode {
    Success = 0x10,
    Failure = 0x11,
}

fn exit_qemu(exit_code: QEMUExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xF4);
        port.write(exit_code as u32);
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print!("Hello, world!\nMy name is {}", "gaylover420");
    loop {}
}