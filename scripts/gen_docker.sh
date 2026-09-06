CONTAINER_NAME=my_gen_proto_and_mocks

repo_root=$(git rev-parse --show-toplevel)

docker build -t $CONTAINER_NAME -f ./gen.Dockerfile .
docker run --rm -v "${repo_root}":/app $CONTAINER_NAME /bin/bash -c "make gen_all"