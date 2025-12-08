use crate::{exit, getchar, ping, putchar, readfile, writefile};

#[no_mangle]
fn main() {
    loop {
        print("> ");
        let mut cmdline: [u8; 128] = [0; 128];
        let mut count = 0;
        loop {
            let ch = getchar() as u8;
            putchar(ch);
            if ch == b'\r' {
                cmdline[count] = b'\0';
                print("\n");
                break;
            } else {
                cmdline[count] = ch;
            }

            count += 1;
            if count == 128 {
                break;
            }
        }
        match core::str::from_utf8(&cmdline[..count]) {
            Ok(s) => {
                if s == "hello" {
                    print("Hello world from shell!\n");
                } else if s == "exit" {
                    exit();
                } else if s == "readfile" {
                    let mut buf: [u8; 128] = [0; 128];
                    readfile("./lorem.txt\0", &mut buf, 128);
                    match core::str::from_utf8(&buf) {
                        Ok(s) => {
                            print(s);
                            print("\n");
                        }
                        Err(_) => print("error"),
                    }
                } else if s == "writefile" {
                    writefile("./lorem.txt\0", b"Hello from virtio\n\0", 128);
                } else if s == "ping" {
                    print("PING 127.0.0.1 (32 bytes of data)\n");

                    let dst_ip: u32 = u32::from_be_bytes([127, 0, 0, 1]);

                    for seq in 0..3 {
                        // TODO: タイマー実装後、送信時刻を記録してRTTを計測する
                        let ret = ping(dst_ip, seq);

                        if ret == 0 {
                            print("Reply from 127.0.0.1: seq=");
                            print_num(seq);
                            // TODO: 実際のRTT（ミリ秒）を表示する
                            print(" time=0.0098ms\n");
                        } else {
                            print("Request timeout for seq=");
                            print_num(seq);
                            print("\n");
                        }

                        // TODO: タイマー割り込み実装後、sleep()システムコールに置き換える
                        // 現在はNOPループによる簡易的なsleep（約1秒）
                        for _ in 0..2000000000 {
                            unsafe { core::arch::asm!("nop") };
                        }
                    }
                } else {
                    print("command not found\n");
                }
            }
            Err(_) => print("command not found\n"),
        }
    }
}

fn print(s: &str) {
    for c in s.bytes() {
        putchar(c);
    }
}

fn print_num(mut n: u32) {
    if n == 0 {
        putchar(b'0');
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;

    while n > 0 {
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        putchar(buf[i]);
    }
}
