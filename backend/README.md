# Backend

The back-end for `smart-fluid-flow-meter`. Written in Rust.

Receives measurements from the meter and stores them in a database.

## Run

The easiest way to run locally is to use:

```
make start
```

This command uses docker compose to start a firestore emulator container and another container running the backend. The backend will use `.env.sample` for configuration.

## Tests

Sadly, firestore requires a service account key to work even when connecting to an emulator, so we need to create one from [google cloud console](https://console.cloud.google.com/iam-admin/serviceaccounts). It's not necessary to give any permissions to the service account. Save the json key to a file named `service-account-key.json` in the same directory as this README file.

To run tests:

```
make test
```

## Production

This section pertains to the official deployment of this service. It can be safely ignored by most people.

## Connecting to production DB

The official deployment uses Firestore for storage. To run the backend locally and connect to the production database we need an Google Cloud JSON key that allows to write access to the correct database.

Put the key in a file named `service-account-key.json` in the same folder as this README file and run using this command:

```
make start-prod
```

## Releasing to production

The official deployment uses Cloud Run. In order to release a new version we first need a service account with the correct permissions:

```
iam.serviceAccounts.actAs
run.services.get
run.services.update
```

Then you need to install [gcloud cli](https://cloud.google.com/sdk/docs/install) and use this command to activate the service account:

```
gcloud auth activate-service-account --key-file=<path to service account>
gcloud auth login <service account>
```

Finally, use this command to release a new version:

```
PROJECT=<gcp project id>
SERVICE_NAME=<gcp service id>
IMAGE=<gcp artifact registry image, including tag>
gcloud run deploy $SERVICE_NAME \
    --project $PROJECT \
    --platform managed \
    --allow-unauthenticated \
    --region us-central1 \
    --image $IMAGE \
    --port 3000
```
