//! Minimal HTTP/1.0 client on top of Kuberve syscalls.
//!
//! Single static buffer (4 KB), not reentrant. For concurrent HTTP
//! requests, spawn separate processes — that's the intended model.

use crate::{
    sys_socket, sys_connect, sys_tcp_send, sys_tcp_recv,
    sys_close, sys_sleep,
};

pub const AF_INET: u64 = 2;
pub const SOCK_STREAM: u64 = 1;

const BUF_SIZE: usize = 4096;
const RECV_CHUNK: u64 = 1024;

static mut HTTP_BUF: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

#[derive(Clone, Copy, Debug)]
pub enum HttpError {
    SocketFail,
    ConnectFail,
    SendFail,
    RecvFail,
    NoBody,
    BufferFull,
}

impl HttpError {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpError::SocketFail  => "socket failed",
            HttpError::ConnectFail => "connect failed (timeout or RST)",
            HttpError::SendFail    => "send failed",
            HttpError::RecvFail    => "recv failed",
            HttpError::NoBody      => "no body in response",
            HttpError::BufferFull  => "response too large (>4 KiB)",
        }
    }
}

/// Perform `GET <path> HTTP/1.0` and return the body.
///
/// Skips HTTP headers (everything up to and including `\r\n\r\n`).
/// Returns a slice into a static buffer — valid until the next call.
///
/// `host` is `[a, b, c, d]` (e.g. `[10, 0, 2, 2]` for QEMU host).
pub fn http_get(
    host: [u8; 4],
    port: u16,
    path: &str,
) -> Result<&'static [u8], HttpError> {
    let ip_u32 = ((host[0] as u32) << 24)
        | ((host[1] as u32) << 16)
        | ((host[2] as u32) << 8)
        | (host[3] as u32);

    // 1. Socket
    let fd = sys_socket(AF_INET, SOCK_STREAM);
    if fd < 0 {
        return Err(HttpError::SocketFail);
    }

    // 2. Connect (3 attempts × up to 2.5 s each = 7.5 s budget).
    //    TCP connect() in kernel already retries SYN internally.
    let mut connected = false;
    for _ in 0..3 {
        if sys_connect(fd, ip_u32, port as u64) >= 0 {
            connected = true;
            break;
        }
        sys_sleep(1);
    }
    if !connected {
        sys_close(fd);
        return Err(HttpError::ConnectFail);
    }

    // 3. Send request.
    let mut req = [0u8; 512];
    let mut req_len = 0usize;
    for &b in b"GET " {
        req[req_len] = b; req_len += 1;
    }
    for &b in path.as_bytes() {
        if req_len >= req.len() - 32 { break; }   // safety
        req[req_len] = b; req_len += 1;
    }
    for &b in b" HTTP/1.0\r\nHost: kuberve\r\nUser-Agent: kuberve-libc/0.14\r\n\r\n" {
        req[req_len] = b; req_len += 1;
    }

    if sys_tcp_send(fd, req.as_ptr(), req_len as u64) < 0 {
        sys_close(fd);
        return Err(HttpError::SendFail);
    }

    // 4. Give the server time to start streaming (QEMU TCG is slow).
    sys_sleep(1);

    // 5. Receive into static buffer.
    let mut total = 0usize;
    let mut idle = 0;
    while total < BUF_SIZE {
        let chunk = core::cmp::min(RECV_CHUNK, (BUF_SIZE - total) as u64);
        let dst = unsafe { HTTP_BUF.as_mut_ptr().add(total) };
        let n = sys_tcp_recv(fd, dst, chunk);
        if n < 0 {
            sys_close(fd);
            return Err(HttpError::RecvFail);
        }
        if n > 0 {
            total += n as usize;
            idle = 0;
        } else {
            idle += 1;
            if idle > 3 { break; }   // no data for 3s, assume end
            sys_sleep(1);
        }
    }
    sys_close(fd);

    if total == 0 {
        return Err(HttpError::NoBody);
    }
    if total == BUF_SIZE {
        return Err(HttpError::BufferFull);
    }

    // 6. Skip HTTP headers.
    let buf = unsafe { core::slice::from_raw_parts(HTTP_BUF.as_ptr(), total) };
    let mut body_start = 0usize;
    let limit = total.saturating_sub(3);
    for i in 0..limit {
        if buf[i] == b'\r' && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r' && buf[i + 3] == b'\n'
        {
            body_start = i + 4;
            break;
        }
    }
    if body_start == 0 || body_start >= total {
        return Err(HttpError::NoBody);
    }

    Ok(unsafe {
        core::slice::from_raw_parts(HTTP_BUF.as_ptr().add(body_start), total - body_start)
    })
}