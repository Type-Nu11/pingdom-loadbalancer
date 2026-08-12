FROM haproxy:3.2-alpine

COPY haproxy/ /usr/local/etc/haproxy/

EXPOSE 80
EXPOSE 8404

CMD ["haproxy", "-f", "/usr/local/etc/haproxy/haproxy.cfg"]