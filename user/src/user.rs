#![no_std]
#![no_main]

mod shell;

use common::{SYS_EXIT, SYS_GETCHAR, SYS_PING, SYS_PUTCHAR, SYS_READFILE, SYS_WRITEFILE};
use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

extern "C" {
    static __stack_top: u32;
}

#[link_section = ".text.start"]
#[unsafe(naked)]
#[no_mangle]
extern "C" fn start() {
    naked_asm!(
        "la sp, {stack_top}",
        "call main",
        "call exit",
        stack_top = sym  __stack_top
    );
}

#[no_mangle]
fn exit() {
    unsafe { syscall(SYS_EXIT, 0, 0, 0) };
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe fn syscall(sysno: u32, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let mut result: u32;

    asm!(
        "ecall",
        in("a0") arg0,
        in("a1") arg1,
        in("a2") arg2,
        in("a3") sysno,
        lateout("a0") result,
    );

    result
}

pub fn putchar(ch: u8) {
    unsafe {
        syscall(SYS_PUTCHAR, ch as u32, 0, 0);
    }
}

pub fn getchar() -> u32 {
    unsafe { syscall(SYS_GETCHAR, 0, 0, 0) }
}

pub fn readfile(filename: &str, buf: &mut [u8], len: u32) -> u32 {
    unsafe {
        syscall(
            SYS_READFILE,
            filename as *const _ as *const u8 as u32,
            buf.as_ptr() as *mut u8 as u32,
            len,
        )
    }
}

pub fn writefile(filename: &str, buf: &[u8], len: u32) -> u32 {
    unsafe {
        syscall(
            SYS_WRITEFILE,
            filename as *const _ as *const u8 as u32,
            buf.as_ptr() as *const u8 as u32,
            len,
        )
    }
}

pub fn ping(dst_ip_be: u32, seq: u32) -> u32 {
    unsafe { syscall(SYS_PING, dst_ip_be, seq, 0) }
}
