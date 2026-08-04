## 📝 Description

Please include a concise summary of the changes made and the motivation behind them. If this PR closes or relates to an open issue, link it below.

Fixes #(issue)

---

## 🛠️ Type of Change

- [ ] 🐛 **Bug fix** (non-breaking change fixing an issue)
- [ ] 🚀 **New feature** (non-breaking change adding functionality)
- [ ] ⚡ **Performance improvement**
- [ ] ♻️ **Refactoring** (code structure improvement with no logic changes)
- [ ] 🛡️ **Security enhancement**
- [ ] 📚 **Documentation update**
- [ ] 🔧 **CI/CD or Tooling update**

---

## 🔒 Security Compliance Checklist

- [ ] No passwords or sensitive inputs are stored or printed in plaintext.
- [ ] Any new credential types use `secrecy::SecretString` and derive `ZeroizeOnDrop`.
- [ ] No raw password strings appear in log calls (`tracing::info!`, `debug!`, `error!`).
- [ ] File permissions and IPC socket security restrictions (`0o600`) are respected.

---

## ✅ Quality Checklist

- [ ] `cargo fmt --all -- --check` passes without formatting changes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` runs without warnings.
- [ ] `cargo test --workspace` passes all unit and integration tests.
- [ ] New code is covered by automated unit/integration tests where applicable.
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/).
