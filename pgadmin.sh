docker run -p 80:80 \
    -e 'PGADMIN_DEFAULT_EMAIL=emerald@example.com' \
    -e 'PGADMIN_DEFAULT_PASSWORD=emerald' \
    --rm dpage/pgadmin4