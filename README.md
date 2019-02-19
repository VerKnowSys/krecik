## Krtecek/ travMole

> Asynchronous, parallel service checker (and reporter), using industry standard libraries: Curl, ngHTTP2 and OpenSSL.


### Author:

Daniel (@dmilith) Dettlaff


### Requirements:

HardenedBSD | Darwin => `Curl_lib` software bundle (`/Software/Curl_lib`), built with all required features enabled by default.
Linux                => `curl-dev`, `openssl-dev`, 'nghttp2-dev'


### Few words about design solutions…

… and especially about current state of linking with shared dynamic libraries
by Cargo on LLVM-driven FreeBSD systems…

To make a long story short - Cargo on FreeBSD/ HardenedBSD/ Linux, doesn't set
proper runtime path (RPATH/RUNPATH in binary header), when shared libraries are
outside of standard /lib:/usr/lib:/usr/local/lib library paths.

There are two quick solutions for this problem - one is `bad`, one is `ugly`.

Bad solution is hacking LD_LIBRARY_PATH shell-env value - and this is
considered to be unethical choice (but still… choice of the many…).

Ugly solution is ugly, but at least solves problem for development time…

NOTE: Krtecek at current stage will use static linking by default.
This means that each release will encapsulate exact versions of:
Curl, OpenSSL and ngHTTP2 libraries - linked directly into travmole binary.


### Features:

Supports all protocols supported by Curl (FILE, FTP, FTPS, GOPHER, HTTP, HTTPS, IMAP, IMAPS, LDAP, LDAPS, POP3, POP3S, RTSP, SMB, SMBS, SMTP, SMTPS, TELNET, TFTP, SFTP, SCP)

HTTP2 used as default protocol, with fallback to HTTP1.1.

TLS1.3, TLS1.2, TLS1.1 as default TLS/SSL protocol versions.

OpenSSL 1.1.1a+ as base for TLS/SSL and domain expiry checks.



### Build fast ("dev" mode):

`bin/build dev`


### Build slow ("release" mode):

`bin/build`


### Run "dev" server:

`bin/run dev`


### Run "release" server:

`bin/run`


### Run tests:


NOTE: If one of servers mentioned above… is started,
      the script mentioned below will do additional
      round of built in tests over HTTP2-Check-API
      implemented by server.

`bin/test`

