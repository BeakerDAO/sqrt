# suft
Scrypto User-Friendly Test package

# TODO
- [X] Make everything in lowercase in test_environment
- [ ] Parse new coins
- [ ] Create tokens
- [ ] Allow multiple arguments return when instantiating a function
- [ ] Allow multiple possible instantiation
- [ ] Deal with returns and expected changes
- [ ] Deal with proofs
- [X] Deal with buckets
- [X] Generate Transaction Manifests
- [ ] Automatic implementation of method trait 



# Usage
Please create a `rtm` directory at the root of the project, rtm files will be written there.
Then run the following command:
```shell
cargo test -- --test-threads=1
```