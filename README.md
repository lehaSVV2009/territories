# territories

> **Active codebase:** [`territories-release/`](territories-release/) (Vite + React + Rust/WASM). Root `src/` and `public/` are the archived legacy CRA app — do not edit them for new features.

Simple board game [Territories](https://lehasvv2009.github.io/territories/)

## Quickstart

```bash
npm install && npm run dev
npm run build && npm run preview
```

If you modify Rust code under `territories-release/territories-wasm/`, rebuild the committed WASM bindings before running Vite:

```bash
npm --prefix territories-release run build:wasm
```

## Legacy Run

```
$ yarn
$ yarn start
```

## Tests

```
$ yarn test
```

## Deployment

Github pages are used by default. See `.travis.yml`.

## Contributing

Help wanted 😄

If you want to contribute - just create issues and/or pull requests and let's discuss it there. No hard rules.

## License

MIT

## Acknowledgments

Great thanks to [Katerina Soroka](https://github.com/KaterinaSoroka) for the inspiration and [Nicolo John Davis](https://github.com/nicolodavis) for the great library [boardgame.io](https://boardgame.io)!
