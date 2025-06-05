# Wall
## What is it?
Simple "wall" without authorization or such useless
stuff. Pure chaos. You say who are you -> 
post message -> anyone see it. Made in pastel 
tones to calm users down and 
not write something sad :)
## Stack
Vanilla js + css + html + rust + axum + 
tokio + tower + serde + anyhow + im tired and it's 3:25 now

## Why is it cool?
idk, for now it saves msgs in ram as vec. When I will
implement something smarter, it will be cool, I promise!

## Usage
```bash
git clone <repolink>

# will work with mock_db
cargo run

# will work with sqlite_db
touch db.sqlite # or any other name 
cargo run --features "sqlite_db"
```
Also you can pass env vars: PORT and DB_FILENAME

## TBD
- [x] storage which is adequate to a problem 
- [x] integrations (implement a trait and, for 
each message, get a thread calling your very useful func)
  - [x] telegram channel posting as an example
- [x] rate limiting (or it'll be more chaotic, then I 
expected even for five people)
- [ ] flake.nix

## LICENSE
I've visited https://choosealicense.com/, and they told me
to pick GNU Public License, I guess. So, read it and use this code if you want
as license tell you..?
