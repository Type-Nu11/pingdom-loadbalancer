FROM haproxy:3.2-alpine

COPY haproxy/haproxy.cfg /usr/local/etc/haproxy/haproxy.cfg

EXPOSE 443
EXPOSE 8404

CMD ["haproxy", "-W", "-db", "-f", "/usr/local/etc/haproxy/haproxy.cfg"]
