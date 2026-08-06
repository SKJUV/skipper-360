use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

/// Structure de tampon sécurisé alignée sur 64 octets (1 ligne de cache CPU)
#[repr(align(64))]
#[derive(Debug, Clone)]
pub struct AlignedSecretBuffer {
    pub data: [u8; 128],
    pub len: usize,
}

impl AlignedSecretBuffer {
    pub fn from_secret(secret: &SecretString) -> Self {
        let raw = secret.expose_secret().as_bytes();
        let len = raw.len().min(127);
        let mut data = [0u8; 128];
        data[..len].copy_from_slice(&raw[..len]);
        data[len] = b'\n';
        Self { data, len: len + 1 }
    }
}

impl Drop for AlignedSecretBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
        speculation_barrier();
    }
}

/// Barrière de sérialisation d'instructions CPU (LFENCE / ISB)
#[inline(always)]
pub fn speculation_barrier() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_lfence();
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        std::arch::asm!("isb sy");
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

/// Expulsion physique des lignes de cache CPU (CLFLUSH)
///
/// # Safety
/// Le pointeur `ptr` doit pointer vers une adresse mémoire valide pour l'expulsion de la ligne de cache.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn flush_cache_line(_ptr: *const u8, _len: usize) {
    speculation_barrier();
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_clflush(_ptr);
    }
    speculation_barrier();
}

/// Activation des protections noyau anti-spéculation (prctl Linux)
pub fn apply_kernel_hardened_prctl() {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        use nix::libc::{prctl, PR_SET_SPECULATION_CTRL, PR_SPEC_DISABLE, PR_SPEC_STORE_BYPASS};
        unsafe {
            // Désactive le Speculative Store Bypass (Spectre-v4) sur x86_64
            let _ = prctl(
                PR_SET_SPECULATION_CTRL,
                PR_SPEC_STORE_BYPASS,
                PR_SPEC_DISABLE,
                0,
                0,
            );
        }
    }
}
