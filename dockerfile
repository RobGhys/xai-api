FROM postgres:16

ENV POSTGRES_DB=app_db
ENV POSTGRES_USER=app_user
ENV POSTGRES_PASSWORD=app_password

COPY init-db.sql /docker-entrypoint-initdb.d/

EXPOSE 5432