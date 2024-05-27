# Basic rust service

## Description

It's a basic service that shows how to work with rust and different rust libraries

## How to run

1. Clone this repo

    ```shell
    git clone <url>
    ```

2. Go to the cloned folder

    ```shell
    cd basic-rust
    ```

3. Copy the [.env.example](.env.example) file and change variables to what you need (Only if you use Taskfile)

    ```shell
    cp .env.example .env
    ```

4. Copy the [config.example.toml](config.example.toml) file and change variables to what you need

    ```shell
    cp config.example.toml config.toml
    ```

5. Run the following command:

    ```shell
    cargo run
    ```

6. Or you can run `Run` configuration if you are using `RustRover` or `VsCode`
7. Or you can run the following taskfile command:

    ```shell
    task run
    ```
