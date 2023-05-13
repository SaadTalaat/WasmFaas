FROM envoyproxy/envoy:v1.26-latest

ARG BUILD_ENV
COPY ./etc/envoy/ /etc/envoy/
COPY /etc/envoy/envoy.${BUILD_ENV}.yaml /etc/envoy/envoy.yaml

