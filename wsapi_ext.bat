set RUST_BACKTRACE=1
curl -s -o - http://10.8.0.1:8080 | ffmpeg -v quiet -i - -f f32le -ac 2 -ar 48000 - | cargo run --release -- -e -s 1 -c 2 -f f32le -r 48000