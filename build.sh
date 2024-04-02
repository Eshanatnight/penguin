docker buildx create --name builder --use
docker buildx build --push --tag ghcr.io/eshanatnight/penguin:latest --platform linux/amd64,linux/arm64 .
