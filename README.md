# Krecik

> Asynchronous, parallel external service checker (and reporter), using industry standard libraries: Curl, ngHTTP2 and OpenSSL.


![krecik](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik.png?raw=true)



# Author:

Daniel ([@dmilith](https://twitter.com/dmilith)) Dettlaff



## Software requirements:

- Rust >= 1.32.0
- Curl >= 7.x
- OpenSSL >= 1.1.1a
- NgHTTP2 >= 1.36.0
- Jq >= 1.5



## Additional build requirements:

- Clang >= 6.x
- Make >= 3.x
- Perl >= 5.x
- POSIX compliant base-system (tested on systems: FreeBSD/ HardenedBSD/ Darwin and Linux)



## Few words about design solutions…

… and especially about current state of linking with shared dynamic libraries
by Cargo on LLVM-driven FreeBSD systems…

To make a long story short - Cargo on FreeBSD/ HardenedBSD/ Linux, doesn't set
proper runtime path (RPATH/RUNPATH in binary header), when shared libraries are
outside of standard /lib:/usr/lib:/usr/local/lib library paths.

There are two quick solutions for this problem - one is `bad`, one is `ugly`.

Bad solution is hacking LD_LIBRARY_PATH shell-env value - and this is
considered to be unethical choice (but still… choice of the many…).

Ugly solution is ugly, but at least solves problem for development time…

NOTE: Krecik at current stage will use static linking by default.
This means that each release will encapsulate exact versions of:
Curl, OpenSSL and ngHTTP2 libraries - linked directly into `krecik` binary.



# Features:

- Asynchronous and multithreaded by default.

- JSON format used for both checks (input) and products (output).

- Uses OpenSSL 1.1.1a+ to provide "TLS-cert expiration check" functionality.

- HTTP2 used as default.


![krecik-ojej](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik_ojej.png?raw=true)



## Caveats. Solutions for potential problems:


Krecik relies on fully featured build of Curl, which is available via Sofin binary-bundle: `Curl_lib`. To install prebuilt "Curl_lib" on supported system:

```bash
_myusername="${USER}"
sudo mkdir "/Software"
sudo chown "${_myusername}" "/Software"
cd "/Software"
curl -O "http://software.verknowsys.com/binary/Darwin-10.11-x86_64/Curl_lib-7.64.0-Darwin-10.11-x86_64.txz"
tar xfJ "Curl_lib-7.64.0-Darwin-10.11-x86_64.txz" --directory "/Software"
```

Prebuilt version of `Curl_lib` bundle is available for systems:

- [Darwin-10.11.x](http://software.verknowsys.com/binary/Darwin-10.11-x86_64/Curl_lib-7.64.0-Darwin-10.11-x86_64.txz)

- [Darwin-10.14.x](http://software.verknowsys.com/binary/Darwin-10.14-x86_64/Curl_lib-7.64.0-Darwin-10.14-x86_64.txz)

- [HardenedBSD-11.x](http://software.verknowsys.com/binary/FreeBSD-11.0-amd64/Curl_lib-7.64.0-FreeBSD-11.0-amd64.zfsx) - NOTE Under HardenedBSD, binary-bundle file is NOT a tar file, but XZ compressed ZFS dataset of software bundle.



## Development:


Lazy mode (using `cargo-watch` + `cargo-clippy`, warnings: enabled, watch awaits for code change for first run):

`bin/devel`


Eager mode (using `cargo-watch` + `cargo-clippy`, warnings: enabled, watch compiles code immediately):

`bin/devel dev`



## Building:


Fast ("dev" mode):

`bin/build dev`


Slow ("release" mode):

`bin/build`



## Running:


Launch "dev" server:

`bin/run dev`


Launch "release" server:

`bin/run`



## Testing:

NOTE: If one of servers mentioned above… is started, the script mentioned below will do additional round of built in tests over HTTP2-Check-API:

`bin/test`



![krecik-build](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik_build.png?raw=true)



# Mapping remote configuration resources:

For now, the only defined remote resource type is: "PongoHost". To configure Pongo API resource, create file: `checks/remotes/yourname.json` with contents:

```JSON
{
    "url": "https://pongo-api.your.domain.tld/api/ping?token=your-secret-token",
    "only_vhost_contains": "services-domain.tld"
}
```

NOTE: If "only_vhost_contains" is "" - no domain filtering is applied (all defined hosts always accepted). If value is set, checker will limit processed checks to only URLs matching specified domain-name (or URL path fragment).



# External JSON resources repositories support:

1. Create new repository with JSON files with definitions of your checks. Check file-format examples can be found in: `checks/tests/*.json`. Commit your checks.

2. Now in `krecik` repository do: `cd krecik/checks`.

3. Clone your checks-resource repository, here I called it "frontends": `git clone git@github.com:my-company-id/krecik-frontends.git frontends`.

4. Start `krecik` web-server in dev mode: `bin/run dev` (starts MUCH faster in dev mode).

5. Use provided WebAPI to perform checks. Examples below.



# WebAPI usage examples

NOTE: early stage, details may change in future!

1. Perform all checks from local "frontends" resource: `curl http://127.0.0.1:60666/check/execute/frontends`

2. Perform only checks defined in a single check-file of local "frontends" resource: "your-name.json": `curl http://127.0.0.1:60666/check/execute/frontends/your-name.json`

3. Perform all checks provided by Pongo remote resource (requires valid mapper configuration per remote resource): `curl http://127.0.0.1:60666/check/execute_remote/remotes`



# Why "Krecik"?

It's been my favorite cartoon… It's a little tribute for mr Zdeněk Miler as well :)



## License

- BSD

- MIT

