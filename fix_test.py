import sys

with open("src/crates/app/src/puzzle_generator.rs", "r") as f:
    content = f.read()

content = content.replace("use super::*;", "use super::*;\n    use crate::solver::solve;")

with open("src/crates/app/src/puzzle_generator.rs", "w") as f:
    f.write(content)
