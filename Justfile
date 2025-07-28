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
