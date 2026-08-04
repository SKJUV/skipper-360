# Guide de Contribution — Skipper 360

Merci d'envisager de contribuer à **Skipper 360** ! Skipper 360 est un projet open-source et communautaire. Nous apprécions grandement toutes les contributions : corrections de bugs, nouvelles fonctionnalités, améliorations de documentation, ou retours d'expérience.

---

## Code de Conduite

En participant à ce projet, vous vous engagez à respecter notre [Code de Conduite](CODE_OF_CONDUCT.md). Veuillez le lire afin de maintenir un environnement accueillant, respectueux et constructif pour tout le monde.

---

## Comment Contribuer ?

### 1. Signaler un Bug

Avant de créer un ticket, vérifiez dans la liste des *Issues* existantes si le problème n'a pas déjà été signalé. Si ce n'est pas le cas, ouvrez un ticket avec :
- Une description claire et concise du comportement observé et du comportement attendu.
- Votre environnement : distribution Linux, version du noyau, environnement de bureau (GNOME, KDE, i3, Sway, etc.).
- Les étapes exactes pour reproduire le problème.
- Les logs pertinents (`~/.config/skipper360/skipper.log`). **Attention : assurez-vous qu'aucun identifiant personnel ne figure dans vos extraits de logs.**

### 2. Proposer une Fonctionnalité

Si vous avez une idée d'amélioration ou de nouvelle fonctionnalité :
1. Ouvrez une *Issue* de type **Feature Request**.
2. Expliquez le cas d'usage concret et la valeur ajoutée pour les utilisateurs.
3. Discutez de la conception avec la communauté avant de commencer à écrire du code.

---

## Environnement de Développement

### Prérequis

- **Rust** 1.75+ et Cargo (via [rustup.rs](https://rustup.rs))
- Dépendances système (sur Debian/Ubuntu) :
  ```bash
 sudo apt install build-essential pkg-config libdbus-1-dev
  ```
- Dépendances système (sur Arch Linux) :
  ```bash
 sudo pacman -S base-devel pkgconf dbus
  ```

### Cloner et Compiler le Projet

```bash
# Cloner le dépôt
git clone https://github.com/skjuve/skipper-360.git
cd skipper-360

# Compiler en mode dev
cargo build

# Vérifier la compilation sans générer de binaire
cargo check --workspace
```

---

## Tests et Qualité du Code

Toute contribution de code doit respecter les standards de qualité de l'écosystème Rust. Avant d'ouvrir une Pull Request, assurez-vous que toutes les commandes suivantes s'exécutent sans erreur ni avertissement :

### 1. Formatage du Code

Le projet utilise `rustfmt`. Formatez votre code avec :

```bash
cargo fmt --all -- --check
```

### 2. Linter Clippy

Aucun avertissement `clippy` n'est toléré sur la branche principale :

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### 3. Exécution des Tests Unitaires et d'Intégration

```bash
cargo test --workspace
```

---

## Règles de Sécurité Non-Négociables

Skipper 360 manipulate des données hautement sensibles. Tout ajout ou modification de code doit strictement se conformer aux principes de sécurité suivants :

1. **Aucun secret en texte brut** : Les mots de passe ne doivent jamais être instanciés sous forme de `String` classique ou `&str`. Utilisez toujours `SecretString` du crate `secrecy`.
2. **Effacement mémoire (`zeroize`)** : Toutes les structures manipulant des secrets doivent dériver `Zeroize` et `ZeroizeOnDrop`.
3. **Absence de trace dans les logs** : Vérifiez qu'aucune macro `tracing` (`info!`, `debug!`, `error!`) n'imprime directement ou indirectement la valeur d'un mot de passe.

---

## Conventions de Commit

Nous suivons les conventions [Conventional Commits](https://www.conventionalcommits.org/) pour garder un historique clair et automatiser la génération des notes de version :

- `feat:` Nouvelle fonctionnalité (ex: `feat(injector): add support for passphrase prompts`)
- `fix:` Correction de bug (ex: `fix(pty): resolve race condition in slave allocation`)
- `docs:` Documentation uniquement (ex: `docs: update quickstart instructions in README`)
- `style:` Changements de formatage du code sans modification de logique
- `refactor:` Refactorisation du code
- `test:` Ajout ou correction de tests
- `chore:` Tâches de maintenance (dépendances, CI, etc.)

---

## Processus de Pull Request (PR)

1. Forkez le dépôt et créez une branche de fonctionnalité depuis `main` :
   ```bash
 git checkout -b feat/nom-de-ma-feature
   ```
2. Écrivez du code propre, documenté et accompagné de tests.
3. Assurez-vous que `cargo test`, `cargo fmt` et `cargo clippy` passent sans soucis.
4. Poussez votre branche sur votre fork et ouvrez une Pull Request vers `main`.
5. Décrivez clairement le but de la PR et faites référence aux issues associées.
6. Répondez aux retours de la revue de code avec bienveillance.

Merci encore pour votre investissement dans **Skipper 360** ! 
