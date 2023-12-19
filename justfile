pack:
    wasm-pack build ./kormir-wasm --dev --weak-refs --target web --scope benthecarman

link:
    wasm-pack build ./kormir-wasm --dev --weak-refs --target web --scope benthecarman && cd kormir-wasm/pkg && pnpm link --global

login:
    wasm-pack login --scope=@benthecarman
