# AURDash 🧭

*The AUR helper that actually helps*

---

## ✨ Overview

**AURDash** is a fast, keyboard-driven terminal UI for managing both **official Arch repositories** and the **AUR** in one place. It focuses on **speed, clarity, and safety**, giving you real-time search, package management tools, and a built-in **security scoring system** to help you make smarter install decisions.

---

## 🚀 Features

### 🔎 Instant Search (Search-as-you-type)

* Real-time package lookup from:

  * Official repositories
  * AUR
* Fuzzy matching for fast discovery
* Zero waiting, zero friction

### 🛡️ Security Score System

Each AUR package is analyzed and given a **security score** based on:

* PKGBUILD inspection
* Maintainer reputation
* Update frequency
* Community signals (votes, popularity)
* Optional AI-assisted review

> Helps you avoid sketchy or unsafe packages without digging manually.

---

### 📦 Package Management

Manage your system without leaving the TUI:

* Install packages (official + AUR)
* Remove packages (with dependency awareness)
* Reinstall packages
* Upgrade system (full sync)
* View detailed package info

---

### 📋 Installed Packages Dashboard

* Clean overview of everything installed
* Quickly:

  * Remove unused packages
  * Reinstall broken ones

---

### ⚡ Performance Focused

* Written for speed and responsiveness
* Minimal resource usage
* Async operations for smooth UX

---

### 🎨 Clean TUI Experience

* Keyboard-first navigation
* Smooth transitions
* Minimal clutter, maximum clarity

---

## 🧠 Why AURDash?

Typical AUR helpers focus only on installation.
**AURDash goes further:**

* Makes package discovery **interactive**
* Adds **security awareness**
* Gives you **full control** over installed packages
* Feels like a **modern dashboard**, not a script wrapper

---

## 🛠️ Installation

```bash
git clone https://github.com/yourusername/aurdash.git
cd aurdash
makepkg -si
```


## 🔐 Security Philosophy

AURDash **does NOT blindly trust AUR packages**.

Instead, it:

* Surfaces risk clearly
* Encourages informed decisions
* Keeps you in control

> You should always review PKGBUILDs — AURDash just makes it easier.

---

## 🤝 Contributing

Pull requests are welcome.
If you have ideas for improving security analysis or UX, open an issue.

---

## 📜 License

MIT License

---

## ⚠️ Disclaimer

AURDash provides **guidance**, not guarantees.
Always verify packages before installing from the AUR.

---

## 💡 Final Note

AURDash is built for people who love control, speed, and clean tooling.
If you live in the terminal, this should feel like home.
