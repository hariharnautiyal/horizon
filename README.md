# Horizon Client

Horizon Client is the client-side component of the Horizon C2 server, a command and control (C2) server for a Rust-based remote access trojan (RAT). It's designed to connect to the server, receive commands, execute them, and send back the results.

## Status

**This project is currently under active development.** Features may be incomplete or subject to change.

## Features

*   **Connects to Horizon Server:** Establishes a connection with the Horizon C2 server.
*   **Command Execution:** Receives and executes shell commands from the server.
*   **File Uploads:** Can upload files from the client machine to the server.
*   **Secure Communication:** Uses JWT for secure communication with the server.
*   **Automatic Registration:** Automatically registers with the server on the first run.

## Getting Started

### Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install)
*   [Cargo](https://doc.rust-lang.org/cargo/)

### Installation

1.  Clone the repository:
    ```sh
    git clone <repository-url>
    ```
2.  Navigate to the project directory:
    ```sh
    cd horizon-client
    ```

### Configuration

Before running the client, you need to set up the following environment variables. Create a `.env` file in the root of the project or set the following environment variables:

```
SERVER_URL=http://127.0.0.1:5487
SERVER_KEY=<Server key from horizon-server>
```

### Running the Client

To start the client, run the following command:

```sh
cargo run
```

The client will start, connect to the server, and wait for commands.

## Project Structure

The project is structured as follows:

```
.
├── Cargo.toml
└── src
    ├── main.rs         # Main application entry point
    └── client.rs       # Client logic for communication with the server
```

## Disclaimer

This tool is intended for educational and research purposes only. The author is not responsible for any misuse or damage caused by this program. Use this software responsibly and only on systems you have explicit permission to access.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the MIT License.
