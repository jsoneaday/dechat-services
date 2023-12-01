image_name="postgres:14-alpine"

echo "clear docker images"
container_ids=$(docker ps -q --filter "ancestor=$image_name")
if [ -z "$container_ids" ]; then
  echo "No container id found using image: $image_name"
else
  echo "Stopping and removing containers using image: $image_name"
  docker stop $container_ids
  docker rm $container_ids
  rm -rf devdb
fi

echo "build new images"
docker compose -f docker-compose.dev.yml up -d --build

echo "run sqlx migrations"
sleep 10
sqlx database create --database-url postgres://dechat_db:dechat_db@localhost:5433/dechat_db
sqlx migrate run --source ./repository/migrations --database-url postgres://dechat_db:dechat_db@localhost:5433/dechat_db

# echo "setup test data"
# psql -h localhost -p 5433 -d dechat_db -U dechat_db -f setup-dev-data.sql

# echo "start running tests"
# cargo test -- --nocapture

# echo "start rust server locally (not docker)"
# cargo run