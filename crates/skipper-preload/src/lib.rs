use nix::libc::{self, c_char, c_int, c_void};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_ACTIVE: AtomicBool = AtomicBool::new(true);

type WriteFn = unsafe extern "C" fn(fd: c_int, buf: *const c_void, count: usize) -> isize;

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: usize) -> isize {
    static mut REAL_WRITE: Option<WriteFn> = None;

    if REAL_WRITE.is_none() {
        let symbol = libc::dlsym(libc::RTLD_NEXT, b"write\0".as_ptr() as *const c_char);
        if !symbol.is_null() {
            REAL_WRITE = Some(std::mem::transmute(symbol));
        }
    }

    let real_write = REAL_WRITE.expect("Impossible de résoudre le symbole write original");

    // Intercepter la sortie stdout (fd 1) ou stderr (fd 2) pour la détection de prompt
    if (fd == 1 || fd == 2) && !buf.is_null() && count > 0 && IS_ACTIVE.load(Ordering::Relaxed) {
        let slice = std::slice::from_raw_parts(buf as *const u8, count);
        if let Ok(text) = std::str::from_utf8(slice) {
            if text.contains("password for")
                || text.contains("Mot de passe")
                || text.contains("Password:")
                || text.contains("passphrase")
            {
                if let Ok(home) = std::env::var("HOME") {
                    let socket_path = format!("{}/.config/skipper360/skipper.sock", home);
                    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
                        let req = serde_json::json!({
                            "command": "inject_prompt",
                            "args": { "text": text },
                            "request_id": "preload"
                        });

                        if stream.write_all(format!("{}\n", req).as_bytes()).is_ok() {
                            let mut reader = BufReader::new(&stream);
                            let mut line = String::new();
                            if reader.read_line(&mut line).is_ok() {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line)
                                {
                                    if let Some(data) = parsed.get("data") {
                                        if let Some(password) =
                                            data.get("password").and_then(|v| v.as_str())
                                        {
                                            let timeout = data
                                                .get("timeout")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(10);

                                            // Attendre le délai de timeout (10s par défaut)
                                            std::thread::sleep(std::time::Duration::from_secs(
                                                timeout,
                                            ));

                                            // Injection synthétique des caractères dans le terminal stdin via TIOCSTI ioctl
                                            let pwd_bytes = format!("{}\n", password);
                                            for byte in pwd_bytes.bytes() {
                                                libc::ioctl(
                                                    0,
                                                    libc::TIOCSTI as _,
                                                    &byte as *const u8,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    real_write(fd, buf, count)
}
