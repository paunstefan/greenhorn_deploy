# greenhorn_deploy

Utility for automatically syncing a Git repo when new changes are pushed to the remote on Github.

It uses Github webhooks to trigger the sync.

## Usage

### Webhooks
First the webhook needs to be configued on Github. It's specific to every repo, and can be found under Settings->Webhooks.

You need to create a new one, for the payload URL set the address where you'll be running the server. 
greenhorn_deploy automatically runs on the `/payload` path, so an example payload URL would be `http://[address]:[port]/payload`.

Select Content type `application/json` and the trigger event to `Just the push event.`.

Save the X-Hub-Signature-256 key, as it will be used when starting greenhorn_deploy.

### Starting the server

To start the server you need to set a few environment variables with information about the
repo and webhook. 

These are:

* `GREENHORN_DEPLOY_SIGNATURE` -> The secret key generated when configuring the webhook.
* `GREENHORN_MAIN_BRANCH` -> The full name of the main branch on Github (default is `refs/heads/main`)
* `GREENHORN_REPO_NAME` -> Name of the repo you want to monitor.

After they are set you can start the program:

```
greenhorn_deploy [address:port] [path_to_repo]
```