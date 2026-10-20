#![no_std]

use core::arch::asm;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

// ============================================================
// ABI: должно совпадать с kernel/src/arch/aarch64/ipc.rs
// SYNC WITH kernel/src/arch/aarch64/ipc.rs
// ============================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Message {
    pub tag: u64,
    pub data: [u64; 7],
}

// ============================================================
// Syscall номера (SYNC WITH kernel/src/arch/aarch64/syscall.rs)
// ============================================================
pub const SYS_WRITE:    u64 = 1;
pub const SYS_EXIT:     u64 = 2;
pub const SYS_SLEEP:    u64 = 3;
pub const SYS_GETPID:   u64 = 4;
pub const SYS_SEND:     u64 = 5;
pub const SYS_RECV:     u64 = 6;
pub const SYS_IRQ_ACK:  u64 = 7;
pub const SYS_TRY_SEND: u64 = 8;
pub const SYS_TRY_RECV: u64 = 9;

// ============================================================
// Syscalls
// ============================================================
pub fn sys_write(fd: u64, buf: *const u8, len: u64) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_WRITE,
            inlateout("x0") fd => ret,
            in("x1") buf, in("x2") len, out("x8") _);
    }
    ret
}

pub fn sys_exit(code: u64) -> ! {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_EXIT,
            in("x0") code,
            options(noreturn));
    }
}

pub fn sys_sleep(ticks: u64) {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_SLEEP,
            inlateout("x0") ticks => _,
            out("x8") _);
    }
}

pub fn sys_getpid() -> u64 {
    let ret: u64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_GETPID,
            lateout("x0") ret, out("x8") _);
    }
    ret
}

pub fn sys_send(target: u64, msg: *const Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_SEND,
            inlateout("x0") target => ret,
            in("x1") msg, out("x8") _);
    }
    ret
}

pub fn sys_recv(msg: *mut Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_RECV,
            inlateout("x0") msg as u64 => ret,
            out("x8") _);
    }
    ret
}

pub fn sys_irq_ack(intid: u64) {
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_IRQ_ACK,
            inlateout("x0") intid => _,
            out("x8") _);
    }
}

pub fn sys_try_send(target: u64, msg: *const Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_TRY_SEND,
            inlateout("x0") target => ret,
            in("x1") msg, out("x8") _);
    }
    ret
}

pub fn sys_try_recv(msg: *mut Message) -> i64 {
    let ret: i64;
    unsafe {
        asm!("mov x8, {n}", "svc #0",
            n = const SYS_TRY_RECV,
            inlateout("x0") msg as u64 => ret,
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

static HEAP_NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct BumpAlloc;

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut next = HEAP_NEXT.load(Ordering::Relaxed);
        if next == 0 { next = HEAP_START; }
        let align = layout.align().max(8);
        let aligned = (next + align - 1) & !(align - 1);
        let end = match aligned.checked_add(layout.size()) {
            Some(e) => e,
            None => return ptr::null_mut(),
        };
        if end > HEAP_END { return ptr::null_mut(); }
        HEAP_NEXT.store(end, Ordering::Relaxed);
        aligned as *mut u8
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
            while (*UART_FR & (1 << 5)) != 0 {}
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