# Build for the web

1. Make sure you have both `Rust` and `npm` (which should include `npx`) installed.

2. Run

```console
$ npm install
```

3. Run

```console
$ npx wasm-pack build ".." --target web --out-name web --out-dir web/pkg
```

4. Run

```console
$ npm run serve
```

5. Open `http://localhost:8080` in a browser
