# Krecik

> Asynchronous, parallel external service checker (and reporter), using industry standard libraries: Curl, ngHTTP2 and OpenSSL.


![krecik](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik.png?raw=true)



# Author:

Daniel ([@dmilith](https://twitter.com/dmilith)) Dettlaff



# Features:

- Asynchronous and multithreaded by default.

- JSON format used for both checks (input) and products (output).

- Uses OpenSSL 1.1.1a+ to provide "TLS-cert expiration check" functionality.

- HTTP2 used as default.



## Software requirements:

- Rust >= 1.50.0
- Curl >= 7.x
- OpenSSL >= 1.1.1a
- NgHTTP2 >= 1.36.0



## Additional build requirements:

- Clang >= 6.x
- Make >= 3.x
- Cmake >= 3.16
- Perl >= 5.x
- POSIX compliant base-system (tested on systems: FreeBSD/ HardenedBSD/ Darwin and Linux)



![krecik-ojej](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik_ojej.png?raw=true)



# Configuration:

By default Krecik looks for configuration under:

- /etc/krecik/krecik.conf
- /Services/Krecik/service.conf
- /Projects/krecik/krecik.conf
- krecik.conf

## Krecik dynamic configuration file format:

```json
{
    "log_file": "/var/log/krecik.log",
    "log_level": "INFO",
    "success_emoji": ":krecik-success:",
    "failure_emoji": ":krecik-failure:",
    "ok_message": "All services are UP as they should.",
    "notifiers": [
    {
        "name": "notifier-name",
        "notifier": "https://hooks.slack.com/services/1111111111/222222222/3333333333333"
    },
    {
        "name": "notifier-other-name",
        "notifier": "https://hooks.slack.com/services/1111111111/222222222/3333333333333"
    }
  ]
}
```

Fields explanation:

- `ok_message` - Notification message that will be sent (per notifier) when all checks are successful.

- `log_file` - Krecik log file location.

- `notifiers` - List of Slack notifiers used by each Check definition by name.


## Fully featured Krecik check file example:

```json
{
    "domains": [
        {
            "name": "some-page.com",
            "expects": [
                {
                    "ValidExpiryPeriod": 10
                }
            ]
        },
        {
            "name": "some-other-domain.com",
            "expects": [
                {
                    "ValidExpiryPeriod": 90
                }
            ]
        }
    ],
    "pages": [
        {
            "url": "https://some-page.com/",
            "expects": [
                {
                    "ValidAddress": "https://some-page.com/after/for/example/302/redirect"
                },
                {
                    "ValidCode": 200
                },
                {
                    "ValidContent": "Some content"
                },
                {
                    "ValidContent": "<title"
                },
                {
                    "ValidContent": "and this thing"
                },
                {
                    "ValidLength": 100000
                }
            ],
            "options": {
                "timeout": 15,
                "connection_timeout": 30,
                "verbose": false,
                "ssl_verify_host": true,
                "ssl_verify_peer": true,
                "follow_redirects": true,
                "headers": [
                    "zala: takiheder",
                    "atala: header123",
                    "oitrala: 1"
                ],
                "cookies": [
                    "ala: 123",
                    "tala: aye sensei",
                    "trala: 6"
                ],
                "agent": "Krtecek-Underground-Agent",

                "method": "POST",
                "post_data": [
                    "some: value",
                    "{\"more\": \"data\"}"
                ]
            }
        },
        {
            "url": "http://google.com/fdgrtjkgengjkdfnglksfdgsdfg",
            "expects": [
                {
                    "ValidCode": 404
                }
            ]
        },
        {
            "url": "http://rust-lang.org/",
            "expects": [
                {
                    "ValidCode": 200
                }
            ]
        }
    ],
    "notifier": "notifier-name"
}
```


## Default expectations:

- Domain check expectation: `ValidExpiryPeriod(14)` (each domain has to be valid for at least 14 days).

- Page check expectations: `ValidCode(200)` (http error code is 200) + `ValidLength(128)` (content length is at least 128 bytes long) + `ValidContent("body")` (content contains "body")


# Development:


## Build debug version:


Lazy developer mode (using `cargo-watch` + `cargo-clippy`, warnings: enabled, watch awaits for code change for first run):

`bin/devel`


Eager developer mode (using `cargo-watch` + `cargo-clippy`, warnings: enabled, watch compiles code immediately):

`bin/devel dev`



## Build release version:


`bin/build`



## Run:


Launch "dev" server:

`bin/run dev`


Launch "release" server:

`bin/run`



## Test:

NOTE: If one of servers mentioned above… is started, the script mentioned below will do additional round of built in tests over HTTP2-Check-API:

`bin/test`



![krecik-build](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik_build.png?raw=true)



# Mapping remote configuration resources:

For now, the only defined remote resource type is: "PongoHost". To configure Pongo API resource, create file: `checks/remotes/yourname.json` with contents:

```JSON
{
    "url": "https://pongo-api.your.domain.tld/api/ping?token=your-secret-token",
    "notifier": "notifier-id"
}
```


# External JSON resources repositories support:

1. Create new repository with JSON files with definitions of your checks. Check file-format examples can be found in: `checks/tests/*.json`. Commit your checks.

2. Now in `krecik` repository do: `cd krecik/checks`.

3. Clone your checks-resource repository, here I called it "frontends": `git clone git@github.com:my-company-id/krecik-frontends.git frontends`.

4. Start `krecik` web-server in dev mode: `bin/run dev` (starts MUCH faster in dev mode).



# Build requirements for svdOS systems:

For svdOS (custom HardenedBSD x86_64) servers using Sofin:

Install build requirements with:

`s i Openssl Rust Perl Make`

then publish bundles settings to the environment with:

`s env +Openssl +Rust +Perl +Make`

After build bring back dynamic env setup with:

`s env reset`


![krecik-build](https://github.com/dmilith/krecik/blob/master/src/imgs/krecik_dyrygent.png?raw=true)



# Why "Krecik"?

It's been my favorite cartoon… It's a little tribute for mr Zdeněk Miler as well :)



## License

- BSD

- MIT

