# Shrike Log Viewer 🪺

Shrike Log Viewer is an application that reads logs from `stdout.log` and `stderr.log` files and transmits them via WebSockets to a web interface that displays the logs in real-time with a CLI terminal style.

## Requirements

- Rust
- Cargo

## Installation

1. Clone the repository:🪺

    ```sh
    git clone https://github.com/your-username/shrike.git
    cd shrike
    ```

2. Install the dependencies:

    ```sh
    cargo build
    ```

## Running

1. Run your app with
  ```sh
  node app.js 1>../stdout.log 2>../stderr.log
  ```

2. Run the server:

    ```sh
    cargo run
    ```

3. Open your browser and navigate to:

    ```sh
    http://127.0.0.1:1312/
    ```

## Usage

The application reads the `stdout.log` and `stderr.log` files and transmits the logs via WebSockets. The web interface displays the logs in real-time with a CLI terminal style.

## Project Structure

- `src/main.rs`: Main server code that reads the logs and transmits the messages via WebSockets.
- `src/index.html`: Web interface that displays the logs in real-time.
