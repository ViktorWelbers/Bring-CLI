# Bring! App - CLI

Welcome to the Bring! App Command Line Interface (CLI) for Windows, a convience tool.
This CLI empowers you to effortlessly manage your shopping list and recipes directly from the command line. 
Its standout feature is the ability to seamlessly integrate recipes into your shopping list, simplifying your meal
planning and grocery shopping.

This was created because I got tired of remembering recipes and manually adding the ingredients to my shopping list.
I wanted to be able to add recipes to my shopping list with a single command, instead of having to manually input each
on my phone.

## Disclaimer

This is not an official Bring! App product. It is a third-party tool that uses the Bring! App API to manage your
shopping
list and recipes. The Bring! App API is not officially supported by Bring! Labs AG, the company behind the Bring! App.
This tool is not affiliated with Bring! Labs AG in any way.

It is also probably against the Bring! App's terms of service to use this tool. Use at your own risk.

(It's also my first Rust project, so it's probably not the best code you've ever seen.)

## Installation

To get started, either download it from the current [release](https://github.com/ViktorWelbers/Bring-CLI/releases/tag/v0.0.1) or build it from source:

1. Clone the project.
2. Have [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed
3. Build the project using Cargo: `cargo build --release`.
4. Copy the generated executable to a folder that is included in your system's PATH environment variable.

## Usage

### Login

Before you can start using the Bring! App CLI, you need to log in to your Bring! account. Follow these steps to
authenticate:

1. Retrieve your authentication token by inspecting the network traffic in your web browser's developer tools. Look for
   the `Authorization` header in the request to `https://api.getbring.com/rest/bringlists`. Copy the token (excluding
   the `Bearer` prefix).
2. Obtain the UUID of the shopping list you want to manage. Look for the `listUuid` field in the response of the network
   call to `lists` in your browser's developer tools.

Once you have both the token and the list UUID, run the following commands to set up your credentials:

- To log in, just execute `bring` and paste your token and list UUID when prompted.
- To change your token or list UUID, use the `bring edit-authtoken` or `bring edit-list-uuid` commands, respectively.

Your authentication details are securely stored in a configuration file located at `C:\ProgramData\Bring\kv.db`.

### Commands

#### `bring add <item>`

Add an item to your shopping list.

#### `bring remove <item>`

Remove an item from your shopping list.

#### `bring list`

List all the items currently on your shopping list.

#### `bring recipe <command>`:

The `bring recipe` command group allows you to manage recipes efficiently:

- `bring recipe store <recipe>`: Save a recipe in the `C:\ProgramData\Bring\kv.db` file.
- `bring recipe add <recipe>`: Add the ingredients of a recipe to your shopping list.
- `bring recipe list`: View a list of all stored recipes in the `recipes` folder.
- `bring recipe remove <recipe>`: Remove all ingredients of a specific recipe from your shopping list.
- `bring recipe delete <recipe>`: Delete a recipe from the `C:\ProgramData\Bring\kv.db` file.

Enhance your meal planning and shopping experience with the Bring! App Windows CLI.
Enjoy the convenience of managing your shopping list and recipes seamlessly from the command line:.

## TODO

- [x] Add support for adding and removing items from the shopping list.
- [x] Add support for listing all items on the shopping list.
- [x] Add support for adding and removing recipes from the shopping list.
- [ ] Add unit tests.
- [ ] Add Generative AI support to parse recipes from websites and store them.
- [ ] Add Support for Whisper to add to list via voice commands.
- [ ] Add support for login via email and password.
- [ ] Add support for login via Google.
- [ ] Improve the way you can enter ingredients for recipes. (e.g. `bring recipe add "1 cup of flour"`)
- [ ] Linux support.
