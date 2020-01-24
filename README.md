# Enmass

## Setup

### Credentials

- Create a project at on the [Google Cloud Platform](https://console.cloud.google.com)
- Go the "Credentials" in the side bar
- Click "Create Credentials" then "OAuth client ID"
- Choose "Other" as the "Application Type"
- Choose a name for client (e.g. "enmass")
- Copy the client ID and secret
  - Environment variables `CLIENT_ID` and `CLIENT_SECRET` respectively
- Click on the created client under "OAuth 2.0 Client IDs"
- Click "Download JSON"
  - Keys of interest
    - `auth_uri`
    - `client_id`
    - `client_secret`
