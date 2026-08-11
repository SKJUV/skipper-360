# Skipper 360 Environment Integration Script
# Prepend Skipper shims directory to PATH

SHIMS_DIR="$HOME/.local/share/skipper/shims"
if [ -d "$SHIMS_DIR" ]; then
    case ":$PATH:" in
        *":$SHIMS_DIR:"*) ;;
        *) export PATH="$SHIMS_DIR:$PATH" ;;
    esac
else
    export PATH="$SHIMS_DIR:$PATH"
fi
