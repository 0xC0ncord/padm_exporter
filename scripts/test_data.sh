#!/usr/bin/env bash

token="$(curl -k -X POST "https://$1/api/oauth/token?grant_type=password" -d "username=$2" -d "password=$3" 2>/dev/null | jq -r .access_token)"
curl -k "https://$1/api/variables" -H "Authorization: Bearer ${token}" 2>/dev/null | jq
