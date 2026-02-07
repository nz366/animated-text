# animated-text


Refernce Implementation, Editor of experimental LRC file alternative


# Animated Text Storage Format. 

An experimental format is a compatible fromat that unifies features from various caption lyrics formats. At its core, it prioritizes readability and basic functionality, even at a minimal level of implementation knowledge. It's basically a virtual plain text file sysem with many different files with each of their own independent or co-dependent features. 



## Reserved Characters:

[] is a reserved character to indicate data used by the format reader.

/  are generally used as seperators inside class properties.


### Compared to other formats:

 
| Features     | Animated Text | AMLL | TTML | LRC | WebVTT | Srt |
|--------------|---------------|------|------|-----|--------|-----|
| Editor       | ✅           | [x]  | [x]  | [x] |        |     |
| Plain Text   | ✅           |      |      |     |        |     |
| Lines        | ✅           | [x]  | [x]  | [x] |        |     |
| Syllables    | ✅           | [x]  | [x]  |     |        |     |
| Duets        | WIP           | [-]  | [-]  |     |        |     |
| Idols        | WIP           |      |      |     |        |     |
| File Size    | LOW           | HIGH  | HIGH  | MINIMUM |  LOW      |     |
| Streamable   | WIP           | TBD  | TBD  |     |        |     |
| Positioning  | WIP           | TBD  | TBD  |     |        | [x] |
| Sel. Language | WIP           |    |    |     |        |  |
| **Interop.**     | **Animated Text** | **AMLL** | **TTML** | **LRC** | **WebVTT** | **Srt** |
|               |      |      |      |        |     |              |
| Animated Text  |   Full         |   WIP   |  WIP    |   Full (WIP)  |   WIP      | WIP
| AMLL				   |               |    Full  | Full     |      |        |
| TTML 				   |               |      |  Full    |  Full    |        |
| LRC				   |   Partial     |      |      |      |        |
| WebVTT			   |               |      |      |      |        |     |
| Srt				   |               |      |      |      |        |     |



<!--
https://github.com/amll-dev/applemusic-like-lyrics.git
https://amll-ttml-tool.stevexmh.net/-->


 
## Setting up the development container

### GitHub Codespaces
Follow these steps to open this sample in a Codespace:
1. Click the Code drop-down menu and select the **Open with Codespaces** option.
1. Select **+ New codespace** at the bottom on the pane.

For more info, check out the [GitHub documentation](https://docs.github.com/en/free-pro-team@latest/github/developing-online-with-codespaces/creating-a-codespace#creating-a-codespace).

### VS Code Dev Containers

If you already have VS Code and Docker installed, you can click the badge above or [here](https://vscode.dev/redirect?url=vscode://ms-vscode-remote.remote-containers/cloneInVolume?url=https://github.com/microsoft/vscode-remote-try-rust) to get started. Clicking these links will cause VS Code to automatically install the Dev Containers extension if needed, clone the source code into a container volume, and spin up a dev container for use.

Follow these steps to open this sample in a container using the VS Code Dev Containers extension:

1. If this is your first time using a development container, please ensure your system meets the pre-reqs (i.e. have Docker installed) in the [getting started steps](https://aka.ms/vscode-remote/containers/getting-started).

2. To use this repository, you can either open the repository in an isolated Docker volume:

    - Press <kbd>F1</kbd> and select the **Dev Containers: Try a Sample...** command.
    - Choose the "Rust" sample, wait for the container to start, and try things out!
        > **Note:** Under the hood, this will use the **Dev Containers: Clone Repository in Container Volume...** command to clone the source code in a Docker volume instead of the local filesystem. [Volumes](https://docs.docker.com/storage/volumes/) are the preferred mechanism for persisting container data.

   Or open a locally cloned copy of the code:

   - Clone this repository to your local filesystem.
   - Press <kbd>F1</kbd> and select the **Dev Containers: Open Folder in Container...** command.
   - Select the cloned copy of this folder, wait for the container to start, and try things out!

## Things to try

Once you have this sample opened, you'll be able to work with it like you would locally.

Some things to try:

1. **Edit:**
   - Open `main.rs`
   - Try adding some code and check out the language features.
   - Make a spelling mistake and notice it is detected. The [Code Spell Checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker) extension was automatically installed because it is referenced in `.devcontainer/devcontainer.json`.
   - Also notice that utilities like `rls` and the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension are installed. Tools are installed in the `mcr.microsoft.com/devcontainers/rust` image and Dev Container settings and metadata are automatically picked up from [image labels](https://containers.dev/implementors/reference/#labels).

1. **Terminal:** Press <kbd>ctrl</kbd>+<kbd>shift</kbd>+<kbd>\`</kbd> and type `uname` and other Linux commands from the terminal window.

1. **Build, Run, and Debug:**
   - Open `main.rs`
   - Add a breakpoint (e.g. on line 8).
   - Press <kbd>F5</kbd> to launch the app in the container.
   - Once the breakpoint is hit, try hovering over variables, examining locals, and more.

1. **Install the GitHub CLI using a Dev Container Feature:**
   - Press <kbd>F1</kbd> and select the **Dev Containers: Configure Container Features...** or **Codespaces: Configure Container Features...** command.
   - Type "github" in the text box at the top.
   - Check the check box next to "GitHub CLI" (published by devcontainers) 
   - Click OK
   - Press <kbd>F1</kbd> and select the **Dev Containers: Rebuild Container** or **Codespaces: Rebuild Container** command so the modifications are picked up.

 
## License

Copyright (c) nz366 

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
