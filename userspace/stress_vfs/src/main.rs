#![no_std]
#![no_main]

use libkazah::syscalls::{exit, mkdir, open, write, read, close, yield_cpu};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // 1. Create a directory in TmpFS
    let dir_path = "/testdir";
    let ret = mkdir(dir_path, 0777);
    if ret != 0 {
        exit(-1);
    }

    // 2. Open a new file in that directory
    let file_path = "/testdir/hello.txt";
    // Assuming mode=2 means WRITE | CREATE
    let handle = open(file_path, 2);
    if handle == 0 {
        exit(-2);
    }

    // 3. Write data to the file
    let msg = b"Hello from VFS!";
    let written = write(handle, msg);
    if written != msg.len() as u64 {
        exit(-3);
    }

    // 4. Close the file
    close(handle);

    // 5. Re-open the file for reading (mode=1)
    let handle_read = open(file_path, 1);
    if handle_read == 0 {
        exit(-4);
    }

    // 6. Read the data back
    let mut buf = [0u8; 32];
    let read_len = read(handle_read, &mut buf);
    if read_len != msg.len() as u64 {
        exit(-5);
    }

    // Verify contents
    for i in 0..msg.len() {
        if buf[i] != msg[i] {
            exit(-6);
        }
    }

    // 7. Test DevFS
    let dev_handle = open("/dev/storage1", 1);
    if dev_handle == 0 {
        // May fail if storage1 doesn't exist yet, just ignore for now
    } else {
        close(dev_handle);
    }

    close(handle_read);
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-99);
}
