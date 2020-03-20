# httpd-php-fpm

Lightweight 🍃 fast ⚡ fastcgi php 🐘 proxy and static http server.

## Why another http server for php?

Because nowaday either you go serverless, or you manage distributed services 
in containers. httpd-php-fpm focus on containerized php world.

Well known existing solutions are:
  - [apache + mod-php](https://hub.docker.com/layers/php/library/php/7.4-apache/images/sha256-48dde1707d7dca2b701aa230344c58cb8ec5b0ce8e9dbceced65bec5ccd7d1d0?context=explore): the more official & recommended solution. Unfortunately, 
    this automatically imply a debian based docker image, pretty heavy (>=140Mib) 😭
  - [nginx + php-fpm](https://hub.docker.com/layers/webdevops/php-nginx/alpine/images/sha256-6cada7ab54b149645ea149dde876c70b60a44869c14a360fda328fabb357e2ed?context=explore): the bad news is that to avoid violating the container's single
    root process principle... we also need an init to supervise both nginx & 
    php processes, and the resulting base docker image is still heavy, even with an 
    alpine variant (>=90Mib) 😭
  - [symfony cli](https://github.com/symfony/cli/releases): not recommended 
    for production, not single responsibility (also embed Symfony Cloud utils), 
    closed source and written in Go (hello garbage collector). It also tries to 
    bring the better developer experience so it does some voodoo, trying to make 
    things work whatever your php setup is.

So, here's the budget of this project:
- Lightweight 🍃: base docker image less than 40Mib (compressed). And lowest memory footprint.
- fast ⚡:
  - at least 2k req/s under 2ms at 95 percentile for static files
  - at least 1k req/s under 5ms at 95 percentile for basic php-fpm request

## Inspirations

- [kamasu](https://github.com/hhatto/kamasu)
- [rust-httpd](https://github.com/PritiKumr/rust-httpd/blob/master/src/main.rs)
- [fastcgi-serve](https://github.com/beberlei/fastcgi-serve)

## Compiling

```fish
cargo build -j(nproc) --release
# then run it, for example if you have a www dir:
# ./target/release/httpd-php-fpm -d ./www
```

## Benchmarks

### Static files

current benchmarks, done with [vegeta](https://github.com/tsenart/vegeta) on a 
8 core intel 10th gen CPU.

```shell
➜ echo "GET http://localhost:3000/" | vegeta attack -duration=10s -rate=2000 | tee results.bin | vegeta report
Requests      [total, rate, throughput]         20000, 2000.13, 2000.08
Duration      [total, attack, wait]             10s, 9.999s, 287.523µs
Latencies     [min, mean, 50, 90, 95, 99, max]  219.856µs, 564.652µs, 312.437µs, 813.232µs, 1.763ms, 5.4ms, 21.491ms
Bytes In      [total, mean]                     256240000, 12812.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:20000  
Error Set:
```

### php-fpm

stay tuned!