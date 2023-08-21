#![no_std]
#![no_main]

use user_lib::{exit, get_char, print, println};

#[no_mangle]
pub fn main() {
    println!("---------------------------------");
    println!("executable cmd: ['hello', 'exit']");
    println!("---------------------------------");
    loop {
        print!("> ");
        // Caution! :Currently, the stack provides everything. And it's not freed up,
        // so if you type 8 characters or so, you'll panic 100% because there's not enough kernel stack saved by the context switch.
        let mut cmd_line = [0u8; 228];
        let mut i = 0;

        loop {
            let ch = get_char() as u8;
            print!("{}", core::str::from_utf8(&[ch]).unwrap());

            if i == cmd_line.len() - 1 {
                println!("");
                println!("command line too long");
                break;
            } else if ch == b'\r' {
                println!("");
                let cmd_str = core::str::from_utf8(&cmd_line).unwrap_or("invalid utf-8");
                // match cmd_str {
                //     "hello" => println!("Hello world from shell!"),
                //     "exit" => exit(),
                //     _ => println!("unknown command: {}", cmd_str),
                // };
                if cmd_str.contains("hello") {
                    println!("Hello world from shell!")
                } else if cmd_str.contains("exit") {
                    exit()
                } else {
                    println!("unknown command: {}", cmd_str)
                }
                break;
            } else {
                cmd_line[i] = ch;
                i += 1;
            }
        }
    }
}
