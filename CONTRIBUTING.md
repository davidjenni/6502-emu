# Contributing

This project will welcome contributions and suggestions in the near future.

Once this project is ready to welcome contributions and suggestions:  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit <https://cla.opensource.microsoft.com>.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

## Setting Up Local Dev Environment

Windows, macOS or Linux/WSL:

- install rust via <rustup.rs>:
  - Windows: [rustup-init.exe](https://win.rustup.rs/x86_64)
  - macOS, Linux, WSL: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [git](https://git-scm.com/downloads)
- [VS Code](https://code.visualstudio.com/Download) or your different favorite editor
- recommended VSCode extensions for rust development; full list see [extensions.json](/.vscode/extensions.json):
  - [EditorConfig for VS Code (editorconfig.editorconfig)](https://github.com/editorconfig/editorconfig-vscode)
  - [CodeLLDB](https://github.com/vadimcn/codelldb)
  - [rust-analyzer](https://github.com/rust-lang/rust-analyzer)
  - [pest grammar IDE](https://github.com/pest-parser/pest-ide-tools)
- rust crates/binaries that some extensions depend on (check what is already present with `cargo install --list`):
  - pest LSP server: `cargo install pest-language-server`

## Build, Test and Run

Clone repo, use `cargo` to build, test and run:

```bash
git clone https://github.com/davidjenni/6502-emu.git
cd 6502-emu
cargo test
```

Open VSCode from the root of the repo and hack away!
