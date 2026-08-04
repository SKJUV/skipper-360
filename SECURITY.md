# Security Policy — Skipper 360

The **Skipper 360** team takes the security of our users' credentials and system prompts extremely seriously. As an automated credential injection CLI daemon operating on Linux pseudo-terminals (PTYs), Skipper 360 is designed with a defense-in-depth security model.

---

## Supported Versions

Only the latest release on the `main` branch receives active security updates and vulnerability patches.

| Version | Supported          |
| ------- | ------------------ |
| `main`  | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Security Architecture & Guarantees

Skipper 360 enforces non-negotiable security controls:

1. **Zero Secret Persistence on Disk**: `config.toml` strictly stores non-sensitive metadata and OS keyring key aliases. Passwords are stored exclusively in OS keyrings (GNOME Keyring / KWallet via Secret Service API).
2. **In-Memory Encryption & Memory Zeroization**: Credentials in memory are wrapped in `secrecy::SecretString`. All memory structures carrying sensitive data implement `zeroize::ZeroizeOnDrop` to overwrite memory upon drop.
3. **Memory Lock (`mlock`)**: Pages containing unencrypted credentials are locked in RAM via `nix::sys::mman::mlock` to prevent swapping to disk.
4. **Anti-Process Dump**: The daemon executes `prctl(PR_SET_DUMPABLE, 0)` upon startup to prevent core dumps and `/proc/PID/mem` inspection.
5. **Restricted Unix Socket Permissions**: IPC communication occurs via a Unix Domain Socket restricted strictly to user-only read/write (`0o600`).
6. **Zero Secret Leakage in Logs**: Log outputs (`tracing`) are stripped of any credentials or prompt match contents.

---

## Reporting a Vulnerability

**Do NOT report security vulnerabilities in public GitHub issues.**

If you discover a security vulnerability, flaw in prompt detection memory isolation, or potential credential leak in Skipper 360, please report it privately:

- **Primary Contact**: Contact maintainers directly via GitHub or email specified in repository metadata.
- **Response SLA**: We will acknowledge receipt of your vulnerability report within **48 hours** and provide an estimated timeline for a patch.
- **Public Disclosure**: Security advisories will be published on GitHub Advisories once a fix is released.

### What to Include in Your Report
- Detailed description of the vulnerability and security impact.
- Proof of Concept (PoC) or reproduction steps.
- Linux environment details (Kernel version, OS distribution, Keyring daemon in use).
- Any suggested mitigations or patches if available.

---

## Responsible Disclosure Guidelines

- Give us reasonable time to investigate and patch the issue before making any public disclosure.
- Do not exploit security vulnerabilities to access data without authorization.

Thank you for helping keep Skipper 360 and its community safe! 
