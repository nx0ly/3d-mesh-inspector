cd web

npx wasm-pack build "../rust" --target web --out-name web --out-dir ../web/pkg
yarn run build