FROM node:buster-slim

ARG BUILD_ENV
ENV NODE_ENV=$BUILD_ENV
COPY ./ /worker/
WORKDIR /worker/

CMD ["npm", "start"]
