FROM rust:latest as wasm
WORKDIR /prepams/
RUN cargo install wasm-pack
RUN apt-get update && apt-get install -y jq
COPY shared shared
RUN cd shared && ./build.sh

FROM node:latest
WORKDIR /prepams/backend/
COPY --from=wasm /prepams/shared/pkg /prepams/shared/pkg
COPY backend/package*.json /prepams/backend/
RUN npm install
COPY backend .
EXPOSE 8080

WORKDIR /prepams/evaluation/
COPY evaluation/package*.json /prepams/evaluation/
RUN npm install
COPY evaluation/serve.js /prepams/evaluation/
EXPOSE 8081

WORKDIR /prepams/backend/
CMD node ../evaluation/serve.js & node index.js
