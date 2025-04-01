# PackHub - Decentralized Package Repositories for Linux
[![Build Status](https://github.com/mominul/packhub/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/mominul/packhub/actions?query=branch%3Amain)
[![Rust](https://img.shields.io/badge/rust-1.85.1%2B-blue.svg?maxAge=3600)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![asciicast](/pages/assets/asciinema.svg)](https://asciinema.org/a/ncMOerw3L7RwhTXqA3Ck7T7En)
## 🚀 Install Linux Packages Directly from GitHub!
PackHub dynamically creates virtual Linux package repositories (`apt`, `dnf`, `yum`, etc.) on the fly, pulling directly from GitHub Releases. No need for centralized repositories—just seamless installations.

> [!TIP]
> This project is powered by github 🌟s. Go ahead and star it please! 

## ✨ Features

- **Decentralized Package Management** – Install packages directly from GitHub Releases.
- **Seamless Updates** – Automatically fetches the latest releases and updates your package manager.
- **Smart Versioning** – Detects your system version and selects the most compatible package.
- **Developer Freedom** – No need to maintain separate repositories or rely on a maintainer.
- **User Empowerment** – Get the apps you need instantly, without waiting for repositories or manual downloads.

## 🚀 Usage

To install Linux packages from a GitHub repository using PackHub, the repository must have published Linux packages in its Releases. You'll also need to set up the PackHub repository in your system's package manager.

Replace `OWNER` with the repository owner's name and `REPO` with the repository name. For example, for `https://github.com/sindresorhus/caprine`, use `sindresorhus` as `OWNER` and `caprine` as `REPO`.

If you're unsure, visit [packhub.dev](https://packhub.dev) to generate the correct command for your repository.

### Ubuntu-Based Distributions
```bash
wget -qO- http://packhub.dev/sh/ubuntu/github/OWNER/REPO | sh
```

### Debian-Based Distributions
```bash
wget -qO- http://packhub.dev/sh/debian/github/OWNER/REPO | sh
```

### Fedora
```bash
wget -qO- http://packhub.dev/sh/yum/github/OWNER/REPO | sh
```

### openSUSE
```bash
wget -qO- http://packhub.dev/sh/zypp/github/OWNER/REPO | sh
```

Once the PackHub repository is set up, you can install packages using your system’s package manager (`apt`, `dnf`, `yum`, etc.).

## 🔧 Built With

- [**Rust**](https://www.rust-lang.org/) – Ensuring performance, safety, and concurrency.
- [**Axum**](https://crates.io/crates/axum) – A powerful, async web framework for Rust.
- [**Repology**](https://repology.org/) - Leverages its API to gather `apt` package versions from Ubuntu and Debian repositories.
- [**octocrab**](https://crates.io/crates/octocrab) -  A modern, extensible GitHub API client. 
- [**rpm**](https://crates.io/crates/rpm) -  A pure rust library for building and parsing RPMs.
- [**sequoia-openpgp**](https://crates.io/crates/sequoia-openpgp) - OpenPGP key generation and message signing.

Additional dependencies can be found in the `Cargo.toml` file.

## 🤝 Contributing
We welcome contributions! To get started:
1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature-branch`).
5. Open a Pull Request.

## 🤗 Acknowledgement

Special thanks to **Md. Asadujjaman Noor** ([@gold-4N](https://github.com/gold-4N/)) for providing valuable guidance on OpenPGP key generation and the signing process, as well as facilitating a discount from the hosting provider!



## 📄 License
PackHub is licensed under the [GPLv3 License](LICENSE).

Made with ❤️ by [Muhammad Mominul Huque](https://github.com/mominul) and ✨ [contributors](https://github.com/mominul/packhub/graphs/contributors) ✨!
