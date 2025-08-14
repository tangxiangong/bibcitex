fmt:
    dx fmt && cargo sort -w -o workspace,package && cargo fmt --all

css:
    bunx tailwindcss -i ./input.tailwind.css -o ./assets/tailwind.css

css-watch:
    bunx tailwindcss -i ./input.tailwind.css -o ./assets/tailwind.css --watch

dx-serve:
    dx serve

css-minify:
    bunx tailwindcss -i ./input.tailwind.css -o ./assets/tailwind.css --minify

[parallel]
serve: css-watch dx-serve

icon:
    cd icon-gen && source .venv/bin/activate && uv run main.py

desktop-icon: icon
    cargo tauri icon assets/logo.png -o icons && cd icons && rm -rf android && rm -rf ios
