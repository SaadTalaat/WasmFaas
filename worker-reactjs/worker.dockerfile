FROM node:buster-slim

ARG BUILD_ENV
ENV NODE_ENV=$BUILD_ENV
COPY ./ /worker/
WORKDIR /worker/
RUN mv src/config.$BUILD_ENV.js src/config.js
RUN npm install
RUN npm run build
RUN npm install -g serve

CMD ["serve", "-s", "build"]
