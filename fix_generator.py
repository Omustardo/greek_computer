import sys

with open("src/crates/app/src/puzzle_generator.rs", "r") as f:
    content = f.read()

content = content.replace("use rand::seq::SliceRandom;", "use rand::seq::IndexedRandom;\nuse rand::seq::SliceRandom;")
content = content.replace("use crate::solver::solve;\n", "")

with open("src/crates/app/src/puzzle_generator.rs", "w") as f:
    f.write(content)
