# {{crate}}
<!-- markdownlint-disable-next-line -->
<p align="center"><img src="https://github.com/bisoncorps/waihona/raw/main/assets/waihona.png" alt="mythra" height="100px"></p>

[![Crates.io](https://img.shields.io/crates/v/waihona.svg)](https://crates.io/crates/waihona)
[![Build Status](https://github.com/bisoncorps/waihona/workflows/Build%20and%20Test/badge.svg)](https://github.com/bisoncorps/waihona/actions)
[![Publish Status](https://github.com/bisoncorps/waihona/workflows/Publish%20to%20Cargo/badge.svg)](https://github.com/bisoncorps/waihona/actions)


## Usage

All cloud providers are on by default, to specify a single provider e.g aws:

```toml
[dependencies]
waihona { version = "{{version}}", features = ["aws"], default-features = false }
```

{{readme}}

## License

This project is opened under the [MIT License](./LICENSE) which allows very broad use for both academic and commercial purposes
