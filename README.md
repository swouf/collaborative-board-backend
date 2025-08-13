## Developing

- Run the devcontainer
- Run the migrations
    ```bash
    diesel migration run
    ```
- Start the app

## Building

```bash
docker build -t collaborative-ideation-backend --output=./build .
```


## Deploying

```bash
deploy.sh <path-to-connection-key> <server-ip>
```