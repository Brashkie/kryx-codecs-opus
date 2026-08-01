# IETF / RFC 8251 Opus test vectors

These are the **official** Opus conformance test vectors from RFC 8251
("Updates to the Opus Audio Codec"). They validate that our decoder is
bit-exact with the reference implementation.

## Provenance

- **Source:** https://opus-codec.org/docs/opus_testvectors-rfc8251.tar.gz
  (also mirrored by the IETF; see RFC 8251 §5).
- **Spec:** RFC 8251 (updates RFC 6716).
- **Files used here:** `testvector01.bit` … `testvector12.bit`
  (the `.dec` reference PCM files are not needed for our final-range checks).
- **Not modified.** Committed verbatim for deterministic, offline CI.

## SHA-1 hashes (from RFC 8251, for verification)

```
e49b2862ceec7324790ed8019eb9744596d5be01  testvector01.bit
b809795ae1bcd606049d76de4ad24236257135e0  testvector02.bit
e0c4ecaeab44d35a2f5b6575cd996848e5ee2acc  testvector03.bit
a0f870cbe14ebb71fa9066ef3ee96e59c9a75187  testvector04.bit
9b3d92b48b965dfe9edf7b8a85edd4309f8cf7c8  testvector05.bit
28e66769ab17e17f72875283c14b19690cbc4e57  testvector06.bit
bacf467be3215fc7ec288f29e2477de1192947a6  testvector07.bit
ddbe08b688bbf934071f3893cd0030ce48dba12f  testvector08.bit
3932d9d61944dab1201645b8eeaad595d5705ecb  testvector09.bit
521eb2a1e0cc9c31b8b740673307c2d3b10c1900  testvector10.bit
6bc8f3146fcb96450c901b16c3d464ccdf4d5d96  testvector11.bit
338c3f1b4b97226bc60bc41038becbc6de06b28f  testvector12.bit
```

Verify on your machine (PowerShell):

```powershell
Get-FileHash -Algorithm SHA1 testvector01.bit
```

## How to obtain

```bash
curl -OL https://opus-codec.org/docs/opus_testvectors-rfc8251.tar.gz
tar -zxf opus_testvectors-rfc8251.tar.gz
# copy testvector*.bit into this directory
```

## How they're used

`src/rfc_vectors_tests.rs` parses each `.bit` (the `opus_demo` framing:
`[len:u32 BE][final_range:u32 BE][packet]`), decodes every packet, and asserts
the decoder's `OPUS_GET_FINAL_RANGE` matches the stored value — bit-exact
conformance. If these files are absent the tests skip rather than fail.

## Licensing

The test vectors are published by the IETF / Xiph.Org as part of the Opus
standardization materials. They are redistributed here solely for conformance
testing. See RFC 8251 and https://opus-codec.org/ for terms.
