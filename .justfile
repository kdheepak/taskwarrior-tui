# deletes test data if it exists
clean:
    [ -d tests/data ] && rm -rf tests/data || true

# clones the test data expected by cargo test
setup-tests: clean
    mkdir -p tests
    git clone https://github.com/kdheepak/taskwarrior-testdata tests/data

# run tests ensuring fresh test data each time
test: setup-tests
    cargo test

