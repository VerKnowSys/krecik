# TravMole - "Traversing Mole" - strong typed, asynchronous, parallel service checker (and reporter), using industry standard - Curl library.


## Author:

    Daniel (@dmilith) Dettlaff


## Requirements:

    `Curl` software bundle (`/Software/Curl`), built with Sofin (with all useful features enabled by default which are NOT enabled in 99% of prebuilt Curl versions from ports, fink, brew, rpm or apt builds).


## Features:

* Supports all protocols supported by Curl (file, ftp, ftps, gopher, http, https, imap, imaps, ldap, ldaps, pop3, pop3s, rtsp, smb, smbs, smtp, smtps, telnet, tftp, sftp, scp)

* Supports International Domain Names (libIDN2)

* Supports HTTP2 (ngHTTP2)

* Supports modern and most secure TLS implementation (libreSSL)

* Web API with 'call "anything" from "anywhere"' approach.


## Build:

    `bin/build`


## Run server:

    `cargo run`
