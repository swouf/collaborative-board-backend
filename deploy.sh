#!/bin/bash

set -e

SERVER_PATH="./build/collaborative-ideation-backend"
ENV_PATH="./build/.env"
SERVICE_PATH="./server/collab.service"
SERVICE_NAME="collab"

# --- Usage function ---
usage() {
    echo "Usage: $0 [--init] <path-to-pem> <host-ip>"
    echo "  --init    Run remote initialization (yum install, mkdir, etc.)"
    exit 1
}

# --- Parse arguments ---
INIT_MODE=false
if [ "$1" == "--init" ]; then
    INIT_MODE=true
    shift
fi

if [ $# -ne 2 ]; then
    usage
fi

PEM_PATH="$1"
HOST_IP="$2"
REMOTE_USER="ec2-user"  # Default for Amazon Linux

# --- Check local files ---
if [ ! -f "$PEM_PATH" ]; then
    echo "❌ PEM file not found: $PEM_PATH"
    exit 1
fi
if [ ! -f $SERVER_PATH ]; then
    echo "❌ Missing executable: $SERVER_PATH"
    exit 1
fi
if [ ! -f $ENV_PATH ]; then
    echo "❌ Missing .env file: $ENV_PATH"
    exit 1
fi

# --- Optional initialization ---
if [ "$INIT_MODE" = true ]; then
    echo "🔧 Running remote initialization on $HOST_IP..."
    ssh -i "$PEM_PATH" "$REMOTE_USER@$HOST_IP" <<'EOF'
        set -e
        sudo yum update -y
        sudo yum install -y openssl ca-certificates postgresql15
        mkdir -p ~/app
EOF
fi

# --- Upload files ---
echo "📤 Uploading server executable..."
scp -i "$PEM_PATH" "$SERVER_PATH" "$REMOTE_USER@$HOST_IP:~/app/collaborative-ideation-backend"

echo "📤 Uploading .env file..."
scp -i "$PEM_PATH" $ENV_PATH "$REMOTE_USER@$HOST_IP:~/app/collaborative-ideation-backend.env"

echo "📤 Uploading service file..."
scp -i "$PEM_PATH" $SERVICE_PATH "$REMOTE_USER@$HOST_IP:~/app/collab.service"

echo "🔧 Running remote configuration on $HOST_IP..."
ssh -i "$PEM_PATH" "$REMOTE_USER@$HOST_IP" <<'EOF'
    set -e
    sudo cp ~/app/collaborative-ideation-backend /usr/local/bin/
    sudo chown ec2-user:ec2-user /usr/local/bin/collaborative-ideation-backend
    sudo cp ~/app/collaborative-ideation-backend.env /usr/local/etc/
    sudo chown ec2-user:ec2-user /usr/local/etc/collaborative-ideation-backend.env
    sudo cp ~/app/collab.service /etc/systemd/system/collab.service
    sudo systemctl restart collab.service
EOF

echo "✅ Deployment complete."
