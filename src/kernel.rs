#![no_std]
#![no_main]

mod fs;
mod memory;
mod net;
mod process;
mod sbi;
mod virtio;

use common::{
    ascii_len, println, read_csr, write_csr, TrapFrame, SYS_EXIT, SYS_GETCHAR, SYS_PING,
    SYS_PUTCHAR, SYS_READFILE, SYS_WRITEFILE,
};
use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;
use core::ptr;
use fs::fs_flush;
use process::ProcessManager;
use sbi::{getchar, putchar};

use crate::{
    fs::{fs_init, fs_lookup},
    virtio::Virtio,
};

extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
    static _binary_shell_bin_start: u32;
    static _binary_shell_bin_size: u32;
}

const SCAUSE_ECALL: u32 = 8;

static mut PM: ProcessManager = ProcessManager::new();
static mut VIRTIO: *mut Virtio = core::ptr::null_mut();

#[no_mangle]
fn kernel_main() {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    }

    write_csr!("stvec", kernel_entry);

    let mut virtio = Virtio::new();
    unsafe {
        VIRTIO = core::ptr::addr_of_mut!(virtio);
    }
    unsafe { fs_init(&mut virtio) };

    net::ip::init();
    net::icmp::init();
    println!("[kernel] Network stack initialized");

    unsafe {
        let start = ptr::addr_of!(_binary_shell_bin_start);
        let size = ptr::addr_of!(_binary_shell_bin_size) as usize;

        PM.init();
        PM.create(start, size);
        PM.yield_();
    }

    println!("switched to idle process");

    loop {
        net::ip::process_packets();
        unsafe { asm!("wfi") };
    }
}

#[link_section = ".text.boot"]
#[unsafe(naked)]
#[no_mangle]
extern "C" fn boot() {
    naked_asm!(
        "la sp, {stack_top}",
        "j kernel_main",
        stack_top = sym  __stack_top
    );
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {
        unsafe { asm!("wfi") };
    }
}

#[unsafe(naked)]
#[no_mangle]
extern "C" fn kernel_entry() {
    naked_asm!(
        "csrrw sp, sscratch, sp",
        "addi sp, sp, -4 * 31",
        "sw ra,  4 * 0(sp)",
        "sw gp,  4 * 1(sp)",
        "sw tp,  4 * 2(sp)",
        "sw t0,  4 * 3(sp)",
        "sw t1,  4 * 4(sp)",
        "sw t2,  4 * 5(sp)",
        "sw t3,  4 * 6(sp)",
        "sw t4,  4 * 7(sp)",
        "sw t5,  4 * 8(sp)",
        "sw t6,  4 * 9(sp)",
        "sw a0,  4 * 10(sp)",
        "sw a1,  4 * 11(sp)",
        "sw a2,  4 * 12(sp)",
        "sw a3,  4 * 13(sp)",
        "sw a4,  4 * 14(sp)",
        "sw a5,  4 * 15(sp)",
        "sw a6,  4 * 16(sp)",
        "sw a7,  4 * 17(sp)",
        "sw s0,  4 * 18(sp)",
        "sw s1,  4 * 19(sp)",
        "sw s2,  4 * 20(sp)",
        "sw s3,  4 * 21(sp)",
        "sw s4,  4 * 22(sp)",
        "sw s5,  4 * 23(sp)",
        "sw s6,  4 * 24(sp)",
        "sw s7,  4 * 25(sp)",
        "sw s8,  4 * 26(sp)",
        "sw s9,  4 * 27(sp)",
        "sw s10, 4 * 28(sp)",
        "sw s11, 4 * 29(sp)",
        "csrr a0, sscratch",
        "sw a0, 4 * 30(sp)",
        "addi a0, sp, 4*31",
        "csrw sscratch, a0",
        "mv a0, sp",
        "call handle_trap",
        "lw ra,  4 * 0(sp)",
        "lw gp,  4 * 1(sp)",
        "lw tp,  4 * 2(sp)",
        "lw t0,  4 * 3(sp)",
        "lw t1,  4 * 4(sp)",
        "lw t2,  4 * 5(sp)",
        "lw t3,  4 * 6(sp)",
        "lw t4,  4 * 7(sp)",
        "lw t5,  4 * 8(sp)",
        "lw t6,  4 * 9(sp)",
        "lw a0,  4 * 10(sp)",
        "lw a1,  4 * 11(sp)",
        "lw a2,  4 * 12(sp)",
        "lw a3,  4 * 13(sp)",
        "lw a4,  4 * 14(sp)",
        "lw a5,  4 * 15(sp)",
        "lw a6,  4 * 16(sp)",
        "lw a7,  4 * 17(sp)",
        "lw s0,  4 * 18(sp)",
        "lw s1,  4 * 19(sp)",
        "lw s2,  4 * 20(sp)",
        "lw s3,  4 * 21(sp)",
        "lw s4,  4 * 22(sp)",
        "lw s5,  4 * 23(sp)",
        "lw s6,  4 * 24(sp)",
        "lw s7,  4 * 25(sp)",
        "lw s8,  4 * 26(sp)",
        "lw s9,  4 * 27(sp)",
        "lw s10, 4 * 28(sp)",
        "lw s11, 4 * 29(sp)",
        "lw sp,  4 * 30(sp)",
        "sret"
    );
}

#[no_mangle]
fn handle_trap(f: *mut TrapFrame) {
    let scause = read_csr!("scause");
    let stval = read_csr!("stval");
    let mut user_pc = read_csr!("sepc");

    if scause == SCAUSE_ECALL {
        handle_syscall(f);
        user_pc += 4;
    } else {
        panic!("unexpected trap scause={scause:x}, stval={stval:x}, sepc={user_pc:x}");
    }

    write_csr!("sepc", user_pc);
}

fn handle_syscall(f: *mut TrapFrame) {
    let f = unsafe { f.as_mut().unwrap() };
    match f.a3 {
        SYS_PUTCHAR => putchar(f.a0 as u8),
        SYS_GETCHAR => loop {
            let ch = getchar();
            if ch >= 0 {
                f.a0 = ch as u32;
                break;
            }

            unsafe { PM.yield_() };
        },
        SYS_EXIT => {
            unsafe { PM.exit() };
        }
        SYS_READFILE => {
            let filename = f.a0 as *const u8;
            let filename_len = ascii_len(filename);
            let filename = unsafe {
                core::str::from_utf8(core::slice::from_raw_parts(filename, filename_len - 1))
                    .unwrap()
            };

            let buf = f.a1 as *const u8;
            let mut len = f.a2 as usize;

            let file = if let Ok(f) = fs_lookup(filename) {
                unsafe { f.as_mut().unwrap() }
            } else {
                println!("file not found: {}", filename);
                f.a0 = 0xffff_fffe_u32;
                return;
            };

            if len > file.size {
                len = file.size;
            }

            unsafe { ptr::copy(file.data.as_ptr(), buf as *mut _, len) };
            f.a0 = len as u32;
        }
        SYS_WRITEFILE => {
            let filename = f.a0 as *const u8;
            let filename_len = ascii_len(filename);
            let filename = unsafe {
                core::str::from_utf8(core::slice::from_raw_parts(filename, filename_len - 1))
                    .unwrap()
            };

            let buf = f.a1 as *const u8;
            let mut len = f.a2 as usize;

            let file = if let Ok(f) = fs_lookup(filename) {
                unsafe { f.as_mut().unwrap() }
            } else {
                println!("file not found: {}", filename);
                f.a0 = 0xffff_fffe_u32;
                return;
            };

            if len > file.size {
                len = file.size;
            }

            unsafe { ptr::copy(buf as *mut _, file.data.as_mut_ptr(), len) };
            file.size = len;
            unsafe {
                let virtio = VIRTIO.as_mut().unwrap();
                fs_flush(virtio);
            }
            f.a0 = len as u32;
        }
        SYS_PING => {
            let dst_ip_be = f.a0 as u32;
            let seq = f.a1 as u16;

            let dst = net::ip::IpV4Addr::from_be_u32(dst_ip_be);

            println!("[syscall] ping {} seq={}", dst, seq);

            // Echo Request 送信
            let id = 0x1234u16; // TODO: プロセスIDから生成
            let data = [0u8; 32];

            match net::icmp::send_echo_request(dst, id, seq, &data) {
                Ok(_) => {
                    println!("[syscall] Echo Request sent successfully");
                    net::ip::process_packets();
                    f.a0 = 0;
                }
                Err(e) => {
                    println!("[syscall] Failed to send Echo Request: {:?}", e);
                    f.a0 = 0xffff_ffff;
                }
            }
        }
        _ => panic!("unexpected syscall a3={:x}", { f.a3 }),
    }
}
