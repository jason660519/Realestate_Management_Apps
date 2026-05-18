#!/bin/zsh
set -u

PROJECT_DIR="/Volumes/KLEVV-4T-1/Realestate_Management_Apps"
WEB_URL="http://127.0.0.1:5173"
WEB_PORT="5173"

cd "$PROJECT_DIR" || {
  echo "Project directory not found: $PROJECT_DIR"
  read -r "?Press Enter to close..."
  exit 1
}

echo "Realestate Management Apps"
echo "Project: $PROJECT_DIR"
echo "Web:     $WEB_URL"
echo ""

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is not available in PATH. Install Node.js or open this from a shell with npm configured."
  read -r "?Press Enter to close..."
  exit 1
fi

if [ ! -d "node_modules" ]; then
  echo "node_modules is missing. Installing dependencies with npm install..."
  npm install
  install_status=$?
  if [ "$install_status" -ne 0 ]; then
    echo "npm install failed with exit code $install_status."
    read -r "?Press Enter to close..."
    exit "$install_status"
  fi
fi

LISTENERS="$(lsof -tiTCP:"$WEB_PORT" -sTCP:LISTEN 2>/dev/null || true)"
if [ -n "$LISTENERS" ]; then
  echo "Port $WEB_PORT is already in use:"
  lsof -nP -iTCP:"$WEB_PORT" -sTCP:LISTEN
  echo ""
  read -r "?Stop the current listener and start this project? [y/N] " answer
  case "$answer" in
    [Yy]*)
      echo "$LISTENERS" | xargs kill
      sleep 1
      ;;
    *)
      echo "Canceled. Close the existing listener first, then run this launcher again."
      read -r "?Press Enter to close..."
      exit 1
      ;;
  esac
fi

(
  for attempt in {1..60}; do
    if curl -fsS "$WEB_URL" >/dev/null 2>&1; then
      open "$WEB_URL"
      exit 0
    fi
    sleep 1
  done

  echo "Web page did not become ready within 60 seconds: $WEB_URL"
) &

echo "Starting desktop app. Keep this Terminal window open while developing."
echo "Press Ctrl+C here to stop the app and web dev server."
echo ""

npm run tauri:dev
status=$?

echo ""
echo "Dev launcher exited with code $status."
read -r "?Press Enter to close..."
exit "$status"
