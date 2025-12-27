# emissary
## THIS IS PART OF REPO: https://github.com/altonen/emissary/tree/d537850a1b91addc2bf44ab5249ec1ca65ec4bd9

`emissary` is a lightweight and embeddable [I2P](https://geti2p.net/) router

### Features

* Transports:
  * NTCP2
  * SSU2 (experimental)
* Client protocols:
  * I2CP
  * SAMv3
* Proxies:
  * HTTP
  * SOCKSv5

### Directory layout

* `emissary-core/` - I2P protocol implementation as an asynchronous library
* `emissary-util/` - `emissary-core`-related utilities, such as runtime implementations and reseeder
* `emissary-cli/` - `tokio`-based I2P router implementation


