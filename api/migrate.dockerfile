FROM willsquire/diesel-cli


WORKDIR /
COPY ./migrations/ /migrations
CMD ["migration", "--migration-dir", "migrations", "run"]
