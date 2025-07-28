fmt:
    dx fmt && cargo sort -w -o workspace,package && cargo fmt --all

css:
    ./scripts/tailwindcss-extra  -i ./input.tailwind.css -o ./assets/tailwind.css

css-watch:
    ./scripts/tailwindcss-extra  -i ./input.tailwind.css -o ./assets/tailwind.css --watch

dx-serve:
    dx serve

css-minify:
    ./scripts/tailwindcss-extra  -i ./input.tailwind.css -o ./assets/tailwind.css --minify

[parallel]
serve: css-watch dx-serve
