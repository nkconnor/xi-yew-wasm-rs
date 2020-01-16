export RUST_BACKTRACE=1

# Build the client Bundle
pkill -f "http-server"
set -e
dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
project_dir=${dir}/..
cd $project_dir
echo $project_dir

cd crates/client
cargo web build
wasm-pack build --target web
/usr/local/bin/rollup ./www/main.js --format iife --file ./pkg/bundle.js

# Move it to root dir
rm -r ../../pkg || true
mv pkg ../../pkg

cd ../../

# Serve the files
nohup http-server -p 8085 --cors -c-1 &

# Run the Rust App
echo "Running from $(pwd)"
cargo run
