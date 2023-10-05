<p align="center">
    <img width=300px; src="https://github.com/ViktorWelbers/Bring-CLI/assets/74675555/1d419b9d-e295-47dc-85e5-ecd4f51f5784?raw=true" alt="Sublime's custom image"/>
</p>

# Bring! App - CLI

Welcome to the Bring! App Command Line Interface (CLI) for Windows, a convience tool.
This CLI empowers you to effortlessly manage your shopping list directly from the command line.

This was created because I got tired of picking up my phone every time I wanted to add something to my shopping list.

I wanted to be able to add items to my shopping list with a single command while on my PC.

## Disclaimer

This is not an official Bring! App product. It is a third-party tool that uses the Bring! App API to manage your
shopping
list and recipes. The Bring! App API is not officially supported by Bring! Labs AG, the company behind the Bring! App.
This tool is not affiliated with Bring! Labs AG in any way.

It is also probably against the Bring! App's terms of service to use this tool. Use at your own risk.

(I also just started with Rust, so it's probably not the best code you've ever seen.)

## Installation

To get started, either download it from the
current [release](https://github.com/ViktorWelbers/Bring-CLI/releases/tag/v0.0.1) or build it from source:

1. Clone the project.
2. Have [Rust](https://www.rust-lang.org/tools/install)
   and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed
3. Build the project using Cargo: `cargo build --release`.
4. Copy the generated executable to a folder that is included in your system's PATH environment variable.

## Usage

### Login

Before you can start using the Bring! App CLI, you need to log in to your Bring! account. Follow these steps to
authenticate.

run `bring login` and enter your email and password when prompted. You can use this to change your login details at any
time.

Otherwise, if you haven't logged in yet, you will be prompted to enter your email and password.
Lastly, if your token has expired, you will be prompted to enter your password again.

Your token and your default list are stored in a configuration file located at `C:\ProgramData\Bring\kv.db`.

### Commands

#### `bring add <item> -i <extra-info>`

Add an item to your shopping list. You can add extra information to the item by using the `-i` flag, which will be
displayed below the item in the app.

#### `bring remove <item>`

Remove an item from your shopping list.

#### `bring list`

List all the items currently on your shopping list.

## Roadmap

- [x] Add support for adding and removing items from the shopping list.
- [x] Add support for listing all items on the shopping list.
- [x] Add support for login via email and password.
- [x] Improve the way you can enter ingredients for recipes. (e.g. `bring recipe add "1 cup of flour"`)
- [ ] Integrate the Recipe API from Bring!.
- [ ] Unit tests.