#! /bin/zsh

## Commands 
#   1. docker ps -> list running containers
#   2. docker ps -a -> list all containers    
#   3. docker stop <container_name> -> stop containers
#   4. docker rm <container_name> -> remove containers
#   5. docker start <container_name> -> start containers
#   6. docker exec -it <container_name> /bin/bash -> enter container 
#   7. docker logs <container_name> -> view container logs
#   8. docker restart <container_name> -> restart containers
# ghcr.io/eshanatnight/parseable:test 

printf "Starting Penguin Server...\n"

docker run -idt -p 6188:6188 -e "RUST_LOG=debug" --name "penguin" ghcr.io/eshanatnight/penguin

