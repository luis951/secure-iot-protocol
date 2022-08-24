FROM alpine:latest
RUN apk add bash
EXPOSE 8640
ENTRYPOINT ["node"]