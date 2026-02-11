#!/usr/bin/env fish

echo "Building image for homelab.local..."
docker build --platform linux/amd64 -t linx -f ./api/Dockerfile ./api
docker save -o linx.tar linx
echo "Sending to homelab.local..."
rsync linx.tar homelab.local:oci-images/
echo "Sent to homelab.local! Deleting tar file and exiting."
rm linx.tar
