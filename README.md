 # The SQRT library
 The Scrypto Quick Rtm Testing library is a tool that enables its users to easily generate and use Radix Transaction Manifests to test a Scrypto package.  
The Transaction Manifests are exported in a `rtm` subdirectory located in the package directory.
 # Examples
 A wide variety of usage examples is available in the [test](tests) directory.

# Usage
Once the tests are written, use the following command to launch them:

```shell
cargo test -- --test-threads=1
```

 # TODO for version 1.0
 - [ ] Deal with return of blueprint methods
 - [ ] Allow multiple arguments return when instantiating a function
 - [ ] Allow multiple possible instantiation
 - [ ] Deal with blueprints state
 - [ ] Deal with returns and automatically check how things should have evolved
 - [ ] Automatic implementation of method trait
 - [ ] Better doc

