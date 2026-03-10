import sys

with open("src/crates/app/src/lib.rs", "r") as f:
    content = f.read()

if "pub mod puzzle_generator;" not in content:
    content = content.replace("pub mod solver;", "pub mod solver;\npub mod puzzle_generator;")
    with open("src/crates/app/src/lib.rs", "w") as f:
        f.write(content)
