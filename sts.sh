unset AWS_ACCESS_KEY_ID
unset AWS_SECRET_ACCESS_KEY
unset AWS_SESSION_TOKEN

res=$(aws sts get-session-token --serial-number arn:aws:iam::$1:mfa/$2 --token-code $3)

export AWS_ACCESS_KEY_ID=$(echo $res | jq -r .Credentials.AccessKeyId)
export AWS_SECRET_ACCESS_KEY=$(echo $res | jq -r .Credentials.SecretAccessKey)
export AWS_SESSION_TOKEN=$(echo $res | jq -r .Credentials.SessionToken)
