#![no_std]

use core::arch::asm;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use kuberve_abi::Message;

/// Для tag=1 (WRITE): прочитать len и байты из Message.data.
/// Возвращает срез байт (до 48).
pub fn write_payload(msg: &Message) -> &[u8] {
    let len = msg.data[0] as usize;
    let len = if len > 48 { 48 } else { len };
    unsafe {
        core::slice::from_raw_parts(&msg.data[1] as *const u64 as *const u8, len)
    }
}

pub use kuberve_abi::sys as sys_num;

// ============================================================
// Syscalls
// ============================================================
pub fn sys_write(fd: u64, buf: *const u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::WRITE,
            inlateout("x0") fd => ret,
            in("x1") buf, in("x2") len, out("x8") _);
    }
    ret
}

pub fn sys_exit(code: u64) -> ! {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::EXIT,
            in("x0") code,
            options(noreturn));
    }
}

pub fn sys_sleep(ticks: u64) {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::SLEEP,
            inlateout("x0") ticks => _,
            out("x8") _);
    }
}

pub fn sys_getpid() -> u64 {
    let ret: u64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::GETPID,
            lateout("x0") ret, out("x8") _);
    }
    ret
}

pub fn sys_send(target: u64, msg: *const Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::SEND,
            inlateout("x0") target => ret,
            in("x1") msg, out("x8") _);
    }
    ret
}

pub fn sys_recv(msg: *mut Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::RECV,
            inlateout("x0") msg as u64 => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_irq_ack(intid: u64) {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::IRQ_ACK,
            inlateout("x0") intid => _,
            out("x8") _);
    }
}

pub fn sys_try_send(target: u64, msg: *const Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::TRY_SEND,
            inlateout("x0") target => ret,
            in("x1") msg, out("x8") _);
    }
    ret
}

pub fn sys_try_recv(msg: *mut Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::TRY_RECV,
            inlateout("x0") msg as u64 => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_sbrk(delta: i64) -> u64 {
    let ret: u64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::SBRK,
            inlateout("x0") delta => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_open(path: &str, flags: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::OPEN,
            inlateout("x0") path.as_ptr() as u64 => ret,
            in("x1") path.len() as u64,
            in("x2") flags,
            out("x8") _);
    }
    ret
}

pub fn sys_read(fd: i64, buf: *mut u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::READ,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64,
            in("x2") len,
            out("x8") _);
    }
    ret
}

pub fn sys_close(fd: i64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::CLOSE,
            inlateout("x0") fd as u64 => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_write_file(fd: i64, buf: *const u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::WRITE_FILE,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64,
            in("x2") len,
            out("x8") _);
    }
    ret
}

pub const SYS_SOCKET:   u64 = sys_num::SOCKET;
pub const SYS_BIND:     u64 = sys_num::BIND;
pub const SYS_SENDTO:   u64 = sys_num::SENDTO;
pub const SYS_RECVFROM: u64 = sys_num::RECVFROM;

pub fn sys_socket(domain: u64, kind: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const SYS_SOCKET,
            inlateout("x0") domain => ret,
            in("x1") kind, out("x8") _);
    }
    ret
}

pub fn sys_bind(fd: i64, port: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const SYS_BIND,
            inlateout("x0") fd as u64 => ret,
            in("x1") port, out("x8") _);
    }
    ret
}

pub fn sys_sendto(fd: i64, buf: *const u8, len: u64, dst_ip: u32, dst_port: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const SYS_SENDTO,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64, in("x2") len,
            in("x3") dst_ip as u64, in("x4") dst_port, out("x8") _);
    }
    ret
}

pub fn sys_recvfrom(fd: i64, buf: *mut u8, len: u64, src_ip_out: *mut u8, src_port_out: *mut u16) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const SYS_RECVFROM,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64, in("x2") len,
            in("x3") src_ip_out as u64, in("x4") src_port_out as u64, out("x8") _);
    }
    ret
}

pub fn sys_yield() {
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const sys_num::YIELD, out("x8") _, out("x0") _);
    }
}

pub fn sys_connect(fd: i64, dst_ip: u32, dst_port: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const sys_num::CONNECT,
            inlateout("x0") fd as u64 => ret,
            in("x1") dst_ip as u64, in("x2") dst_port, out("x8") _);
    }
    ret
}

pub fn sys_tcp_send(fd: i64, buf: *const u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const sys_num::TCP_SEND,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64, in("x2") len, out("x8") _);
    }
    ret
}

pub fn sys_tcp_recv(fd: i64, buf: *mut u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0", n = const sys_num::TCP_RECV,
            inlateout("x0") fd as u64 => ret,
            in("x1") buf as u64, in("x2") len, out("x8") _);
    }
    ret
}

pub fn sys_spawn(path: &str, priority: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::SPAWN,
            inlateout("x0") path.as_ptr() as u64 => ret,
            in("x1") path.len() as u64,
            in("x2") priority,
            out("x8") _);
    }
    ret
}

pub fn sys_kill(pid: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::KILL,
            inlateout("x0") pid => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_wait(pid: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const sys_num::WAIT,
            inlateout("x0") pid => ret,
            out("x8") _);
    }
    ret
}

// ============================================================
// Heap (bump allocator)
// HEAP_START/END должны совпадать с mmu.rs L2[4] региона
// ============================================================
pub const HEAP_START: usize = 0x40800000;
pub const HEAP_END:   usize = 0x40A00000;

const CHUNK_SIZE: usize = 64 * 1024;  // 64 KB за раз

static CHUNK_PTR: AtomicUsize = AtomicUsize::new(0);
static CHUNK_END: AtomicUsize = AtomicUsize::new(0);

pub struct BumpAlloc;

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(8);
        let size = layout.size();

        loop {
            let ptr = CHUNK_PTR.load(Ordering::Relaxed);
            let end = CHUNK_END.load(Ordering::Relaxed);

            if ptr != 0 {
                let aligned = (ptr + align - 1) & !(align - 1);
                let new_end = aligned.checked_add(size).unwrap_or(usize::MAX);
                if new_end <= end {
                    CHUNK_PTR.store(new_end, Ordering::Relaxed);
                    return aligned as *mut u8;
                }
            }

            // chunk кончился или не выделен — refill
            let need = (size + align).max(CHUNK_SIZE);
            let old_brk = sys_sbrk(need as i64);
            if old_brk == u64::MAX {
                return ptr::null_mut();
            }
            CHUNK_PTR.store(old_brk as usize, Ordering::Relaxed);
            CHUNK_END.store(old_brk as usize + need, Ordering::Relaxed);
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

// ============================================================
// print! / println!
// ============================================================
pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buf = s.as_bytes();
        while !buf.is_empty() {
            let n = sys_write(1, buf.as_ptr(), buf.len() as u64);
            if n <= 0 { return fmt::Result::Err(fmt::Error); }
            buf = &buf[n as usize..];
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($a:tt)*) => {
        let _ = core::fmt::write(&mut $crate::Stdout, format_args!($($a)*));
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n"); };
    ($($a:tt)*) => {{
        $crate::print!($($a)*);
        $crate::print!("\n");
    }};
}

// ============================================================
// UART MMIO (для драйверов в userspace)
// ============================================================
pub mod uart {
    const UART_BASE: usize = 0x09000000;
    const UART_DR:   *mut u32 = UART_BASE as *mut u32;
    const UART_FR:   *const u32 = (UART_BASE + 0x18) as *const u32;
    const UART_CR:   *mut u32 = (UART_BASE + 0x30) as *mut u32;
    const UART_IMSC: *mut u32 = (UART_BASE + 0x38) as *mut u32;
    const UART_ICR:  *mut u32 = (UART_BASE + 0x44) as *mut u32;
    const FR_RXFE: u32 = 1 << 4;

    pub fn putc(c: u8) {
        unsafe {
            while core::ptr::read_volatile(UART_FR) & (1 << 5) != 0 {}
            core::ptr::write_volatile(UART_DR, c as u32);
        }
    }

    pub fn puts(s: &[u8]) {
        for &b in s {
            if b == b'\n' { putc(b'\r'); }
            putc(b);
        }
    }

    pub fn rx_init() {
        unsafe {
            core::ptr::write_volatile(UART_CR, (1 << 0) | (1 << 8) | (1 << 9));
            core::ptr::write_volatile(UART_ICR, 0x7FF);
            core::ptr::write_volatile(UART_IMSC, 1 << 4);
        }
    }

    pub fn try_read() -> Option<u8> {
        unsafe {
            if (*UART_FR & FR_RXFE) != 0 {
                None
            } else {
                Some((core::ptr::read_volatile(UART_DR) & 0xFF) as u8)
            }
        }
    }
}

pub mod flags {
    pub const O_RDONLY: u64 = 0;
    pub const O_CREAT:  u64 = 1;
    pub const O_TRUNC:  u64 = 2;
    pub const O_APPEND: u64 = 4;
}