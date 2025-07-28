fmt:
    dx fmt && cargo sort -w -o workspace,package && cargo fmt --all

css:
    ./scripts/tailwindcss-extra  -i ./input.tailwind.css -o ./assets/tailwind.css --watch
