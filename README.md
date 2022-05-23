# rq
[![rq-core](https://github.com/protiumx/rq/actions/workflows/rq-core.yml/badge.svg)](https://github.com/protiumx/rq/actions/workflows/rq-core.yml)

`rq` is an interactive HTTP client. It attempts to provide a minimal CLI 
alternative to [vscode-restclient](https://github.com/Huachao/vscode-restclient).

This project was born out of my boredom and curiosity about [PEG](https://en.wikipedia.org/wiki/Parsing_expression_grammar).

## Dependencies

- `pest`: https://github.com/pest-parser/pest
- `reqwest`: https://github.com/seanmonstar/reqwest
- `inquire`: https://github.com/mikaelmello/inquire

## Packages

### `rq-core`

Contains the core functionality: pest grammar and request execution.

### `rq-cli`

CLI application that uses `inquire` to show an interactive prompt.
This is the default target for `cargo` workspaces.
 

## Contributing

PRs are always welcomed. Refer to the [project TODO list](https://github.com/protiumx/rq/projects) for ideas!

## Sponsorship

If you find this project useful you can support my work with:
<a href="https://www.buymeacoffee.com/p3kqm9Z2h" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-red.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" ></a>
