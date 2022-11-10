# suft
Scrypto User-Friendly Test package

# TODO for version 1.0
- [X] Make everything in lowercase in test_environment
- [X] Parse new coins
- [X] Create tokens
- [X] Deal with buckets
- [X] Generate Transaction Manifests
- [ ] Deal with proofs
- [ ] Deal with admin badges
- [ ] Better file system
- [ ] Allow multiple arguments return when instantiating a function
- [ ] Allow multiple possible instantiation
- [ ] Deal with returns and expected changes
- [ ] Automatic implementation of method trait 



# Usage
Please create a `rtm` directory at the root of the project, rtm files will be written there.
Then run the following command:
```shell
cargo test -- --test-threads=1
```