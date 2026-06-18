FROM alpine:latest

COPY ./bin/kanade-server /bin/kanade-server
COPY ./frontend /frontend

RUN chmod +x /bin/kanade-server

CMD [ "/bin/kanade-server" ]
