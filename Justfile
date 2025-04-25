set shell := ["sh", "-c"]

serve:
    -rm -r public/terminal/assets
    cd webpage && npm run build
    cargo run --release