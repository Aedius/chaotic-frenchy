
login:
    shuttle login

precommit:
    cargo fmt
    cargo clippy

deploy: precommit
    shuttle deploy --working-directory discord-bot