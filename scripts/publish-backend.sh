# Exit if there is any error
set -e
# Echo all commands
set -x

# Constants
REGISTRY_URL='us-central1-docker.pkg.dev'
IMAGE_NAME="backend-image-prod"

# Log-in to remote docker as one of the first steps as we have no control
# over the availability of the registry
echo $JSON_KEY > keyfile.json
cat keyfile.json | docker login -u _json_key_base64 --password-stdin https://$REGISTRY_URL

# Get current tag number
LAST_TAG=$(git describe --tags $(git rev-list --tags --max-count=1))

# Build docker image
cd backend
make image-prod

# Publish docker image
docker tag $IMAGE_NAME $REGISTRY_URL/$PROJECT/$REPOSITORY/$IMAGE_NAME:$LAST_TAG
docker push $REGISTRY_URL/$PROJECT/$REPOSITORY/$IMAGE_NAME:$LAST_TAG
