# Placeholder: overwrite with your generated script.
# From repo root (axum-app running on host port 4000):
#   curl -fsS -H "Authorization: Bearer $LIQUIDSOAP_API_TOKEN" \
#     "http://localhost:4000/api/internal/liquidsoap_config" -o docker/radio/radio.liq
# Then: docker compose --profile radio restart liquidsoap
log.level.set(3)
icecast_host = environment.get(default="icecast", "ICECAST_INTERNAL_HOST")
icecast_port = int_of_string(environment.get(default="8000", "ICECAST_INTERNAL_PORT"))
icecast_password = environment.get(default="hackme", "ICECAST_SOURCE_PASSWORD")
output.icecast(%mp3(bitrate=128), host=icecast_host, port=icecast_port, password=icecast_password, mount="/placeholder.mp3", name="Placeholder", mksafe(sine()))
