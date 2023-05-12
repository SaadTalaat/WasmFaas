FROM node:buster-slim

ARG BUILD_ENV
ENV NODE_ENV=$BUILD_ENV
COPY ./ /worker/
WORKDIR /worker/
COPY ./config.$NODE_ENV.js ./config.js

CMD ["npm", "start"]
