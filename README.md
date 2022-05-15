# nomadcoin-rs

## About the project

This project is a rust version of [nomadcoin](https://github.com/nomadcoders/nomadcoin). There's a online class called ['Nomadcoin'](https://nomadcoders.co/nomadcoin/lobby), where you can learn how to create your simple blockchain with golang. Although the main language of the lecture is golang, I studied Rust recently, and wanted to use it in my project to get used to the language. That's why I started this project with Rust.

So, I have two objectives for this project.

1. Understand blockchain technically and how to create it on my own.
2. Get used to Rust so that I can confidently say I am a rustcean even if I'm not that good at it.

## Project History

1. [Created simple blockchain (Lecture #4)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:6c6becb...sjquant:a0fb70d)

   - I created Block and Blockchain struct and some functions.
   - Blockchain hashes data when adding block to its chain.

2. [Created explorer html webpage with rocket (Lecture #5)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:a0fb70d...sjquant:329e473)

   - I used [rocket crate](https://crates.io/crates/rocket) to do this and create rest api in the next section.

3. [Created rest api with rocket (Lecture #6)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:329e473...sjquant:058381f)

   - At this point, I used workspace to support both explorer and rest api. But I decided to remove explorer in the next section and only supports restapi as main entrypoint.
   - In the lecture, nico uses reference of block, but it was somewhat hard to use reference of block in Rust because of lifetime issue. So, I asked a question about it to [Korean Rust Discord](https://discord.gg/uqXGjEz), and I got an answer from someone that it's okay to use clone() and it's not that costly most of the time, and optimization should be done later.

4. [Used database for persisting blockchain snapshot and blocks (Lecture #8)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:8e50427...sjquant:9a334f8)

   - I used [`nut`](https://github.com/Reeywhaar/nut) which is a rust version of Bolt DB. I changed it to PickleDB because it felt like the library has some bug at that time.
   - From this section, blockchain persisted even after server restarted.

5. [Applied 'proof of work' concept to mine block and changed database to PickleDB (Lecture #9)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:fd01e2b...sjquant:c78457b)

6. [Created testutils to drop databse after each test](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:c78457b...sjquant:66808f6)

   - I wanted to drop database resource after each test. I created `DBResource` struct and used `Drop` trait to do that. But I don't think that its interface is good and I want to know a better way to do this in Rust. (Actually I removed this later because I thought saving to `/tmp` is okay, but I still want to know a cool way)

7. [Implemented transaction feature in blockchain network (Lecture #10)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:66808f6...sjquant:d2e6ecc)

   - This was kind of difficult part to make it work correctly, but I managed to do it.
   - In the lecture, golang suffered some race condition situations. but Rust felt safe in those conditions, because in concurreny situation, Rust forces us to use `mutex` lock on the compilation level.

8. [Implemented wallet(auth) feature to verify transaction (Lecture #11)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:d2e6ecc...sjquant:b9a6452)

   - I used [`p256` crate](https://crates.io/crates/p256) to use p256 Elliptic Curves algorithm for cryptography and [`hex` crate](https://crates.io/crates/hex) to create hax string.

9. [Implemented p2p feature for all peers in the block chain to have same blockchain and share their own events (Lecture #12)](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:b9a6452...sjquant:3c49f2e)

   - I used SSE (Server Sent Event) to implement this feature instead of websocket, because `Rocket` didn't support websocket at the time. I implemented this feature, but did somewhat little dirty things (like sleep 1 sec to give time to ensure inter-connection)
   - I used [`tokio` crate](https://crates.io/crates/tokio) to support async concurrency.
   - I learned `Arc` to control lifetime in concurreny situation.

10. [Refactored some codes about p2p feature](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:3c49f2e...sjquant:2253846)

    - Used the same parameters for event handling to enhance its adaptibility in the future.

11. [Refactored entire code and test](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:2253846...sjquant:2702dc3)

    - Created repository layer and used test double(stub) for testing blockchain.
    - I learned that I should use `Box` smart pointer to abstract a trait like repository patteren.
    - I learned that I should use `Mutex` (Or `RefCell`) to borrow immutables as mutables. I used this in `TestRepository` and `PickleDBRepository`, which are required to borrow as mutables to write to db, whereas some database library don't have to do that.

12. [Create dockerfile and docker-compose and refactored http file](https://github.com/sjquant/nomadcoin-rs/compare/sjquant:2702dc3...sjquant:ac3501b)

    - I refered to [this blog post](https://dev.to/rogertorres/first-steps-with-docker-rust-30oi)

## Future Plan

This is just my toy project, so I have no plan to add new features, or maintain it. But I want to get feedback about my code like what I'm doing wrong in Rust, and how to improve my rust code. I want to be a good rustcean.

## Final Comment

It was very meaningful to do this project. I can confidently say that I kind of achieved both objectives. (Not that good though). I understood the basics of blockchain and got used to program with Rust.
