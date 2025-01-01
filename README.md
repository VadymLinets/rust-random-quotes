# Quotes service

## Description

It's a service that shows random quotes, allows to like them and show quote that is pretty similar to a specified one

## Additional programs

1. [Taskfile](https://taskfile.dev/installation/) (Optional)
2. docker-compose or podman-compose
3. [Postman](https://www.postman.com/downloads/) or [Yaak](https://yaak.app/download)

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
    cp ./configs/config.example.toml ./configs/config.toml
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

## Run in container mode

1. Create a special container config file and change variables to what you need:

   ```shell
    cp ./configs/config.example.toml ./configs/config.container.toml
    ```

2. Run the following command:

   ```shell
   cd containers && docker-compose -f database.yml up -d
   ```

3. Or you can run the following taskfile command:

   ```shell
   task crun
   ```
