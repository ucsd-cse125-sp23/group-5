<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a name="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->

<br />
<div align="center">
<h3 align="center">As the Wind Blows 💨</h3>

  <p align="center">
Project for CSE 125 Spring 2023, Group 5
    <br />
    <a href="https://cse125.ucsd.edu/2023/cse125g5/"><strong>Homepage »</strong></a>
    <br />
    <br />
  </p>
</div>

<!-- ABOUT THE PROJECT -->
As the Wind Blows is a 3D multiplayer game, created with Rust, WGPU, and a touch of love. We built the game engine from
scratch, providing a fresh gameplay experience accompanied by a visually appealing art style. Set in a world of floating
sky islands, this immersive battle game invites you and your friends to enjoy together. Come and discover the charming
universe of As the Wind Blows!


<!-- GETTING STARTED -->

## Getting Started

To get started on contributing to the project, follow the steps below.

### Prerequisites

* Rust (install using [rustup](https://rustup.rs/))
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Building

1. Clone the repo
    ```sh
    git clone https://github.com/ucsd-cse125-sp23/group-5.git
    ```
2. Build the project
    ```sh
    cargo build
    ```
3. Run the server and client(s)
    1) In debug mode(running locally)
    ```sh
    cargo run --features debug --bin server # start the server
    cargo run --features debug --bin client # start a client
    ```
    2) In production mode
   ```sh
    cargo run --features prod --bin server # start the server
    cargo run --features prod --bin client # start a client
    ```

<!-- Testing -->

## Testing

All the unit tests are attached in the src directories in each file with the code that they’re testing.
The convention is to create a module named tests in each file to contain the test functions and to annotate the module
with `cfg(test)`.

1. Run all unit tests
    ```sh
    cargo test
    ```
2. Run tests specific to a crate
    ```sh
    cargo test -p [client|server|common]
    ```
3. Run tests specific to a module
    ```sh
    cargo test -p [client|server|common] -- [module_name]
    ```

<!-- Documentation -->

## Documentation

Documentation is generated using `cargo doc`. The documentation is generated in the `target/doc` directory.

1. Generate documentation and open it in a browser
    ```sh
    cargo doc --open
    ```

<!-- PROJECT STRUCTURE -->

## Project Structure

This project follows a monorepo architecture, where the client, server, and common libraries reside in a single
repository.
There are 3 crates in this project: `client`, `server`, and `common`.

- `client`: This folder contains the code and resources for the game client. The client is responsible for rendering the
  game, handling user input, and communicating with the server.
- `server`: This folder contains the code and resources for the game server. The server is responsible for managing game
  state, handling client connections, and processing game events.
- `common`: This folder houses the shared Rust library that contains code and resources used by both the client and
  server. This can include shared data structures and and utility functions.

<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<!-- Team -->

## Team

- Yunxiang Chi
- Esa Hammado
- Binghong Li
- Xiyan Shao
- Alan Wang
- Shuhua Xie
- Lingye Zhuang

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/github_username/repo_name.svg?style=for-the-badge

[contributors-url]: https://github.com/github_username/repo_name/graphs/contributors

[forks-shield]: https://img.shields.io/github/forks/github_username/repo_name.svg?style=for-the-badge

[forks-url]: https://github.com/github_username/repo_name/network/members

[stars-shield]: https://img.shields.io/github/stars/github_username/repo_name.svg?style=for-the-badge

[stars-url]: https://github.com/github_username/repo_name/stargazers

[issues-shield]: https://img.shields.io/github/issues/github_username/repo_name.svg?style=for-the-badge

[issues-url]: https://github.com/github_username/repo_name/issues

[license-shield]: https://img.shields.io/github/license/github_username/repo_name.svg?style=for-the-badge

[license-url]: https://github.com/github_username/repo_name/blob/master/LICENSE.txt

[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555

[linkedin-url]: https://linkedin.com/in/linkedin_username

[product-screenshot]: images/screenshot.png
