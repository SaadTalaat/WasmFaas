FROM envoyproxy/envoy:v1.26-latest

COPY ./etc/envoy/envoy.yaml /etc/envoy/envoy.yaml
