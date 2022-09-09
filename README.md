## Blobber

blobber packs arbitrary data into [proto-danksharded transactions](https://www.eip4844.com/).

It packs data near-optimally by fiddling with the bits, which allows fitting 254 bits into each field element, compared to the naive "fit 31 bytes to field element" strategy. This allows us to pack six extra bits per field element.

Currently the code can only pack data into blobs. Next step is [to submit that data to geth](https://github.com/asn-d6/blobber/blob/main/blobber.py) using JSON-RPC.

~Under active development.~
