# crunkurrent

A Lil Jon-inspired multitasking tool.

![crunk ain't dead](./liljon.jpg)

crunkurrent lets you run tasks concurrently:

```
$ cr \
    --cmd "npm run dev" \
    --cmd "cd api && flask run"
71170    ├ [started 'npm run dev']
71173    ├ [started 'cd api && flask run']
71170    │ ready - started server on 0.0.0.0:3000, url: http://localhost:3000
71170    │ event - compiled client and server successfully in 613 ms (140 modules)
71173    │  * Debug mode: off
71173    │  * Running on http://127.0.0.1:5000
...
```

You can think of it kinda like docker-compose but for arbitrary shell commands instead of containers.

## Installation

crunkurrent is available as a Nix flake. [Install Nix](https://nixos.org/download.html) and [enable flakes](https://serokell.io/blog/practical-nix-flakes). Then you can play around with crunkurrent with

```
nix shell github:samuela/crunkurrent
```
