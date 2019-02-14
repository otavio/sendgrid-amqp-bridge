#!/bin/sh

echo $CONFIG | base64 -d > config.yaml

/usr/local/bin/sendgrid-amqp-bridge --config config.yaml $EXTRA_ARGS

