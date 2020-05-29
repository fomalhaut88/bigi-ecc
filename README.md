# bigi-ecc

**bigi-ecc** is a Rust library for [elliptic-curve cryptography](https://en.wikipedia.org/wiki/Elliptic-curve_cryptography). It contains the most popular elliptic curves (Weierstrass curve, Montgomery curve, Edwards curve) and algorithms to encrypt and decrypt data, to build signatures and to map data blocks. Also there are several certain elliptic curves to import. The library is built for Rust Nightly strictly.

Available curve types:

1. [Weierstrass curve](https://en.wikipedia.org/wiki/Elliptic_curve)
2. [Montgomery curve](https://en.wikipedia.org/wiki/Montgomery_curve)
3. [Edwards curve](https://en.wikipedia.org/wiki/Edwards_curve)

Implemented algorithms:

* [ElGamal encryption](https://en.wikipedia.org/wiki/ElGamal_encryption)
* [ECDSA](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm)
* Mapping

Curves:

* Secp256k1
* Fp254BNb
* [Curve25519](https://en.wikipedia.org/wiki/Curve25519)
* Curve1174

**bigi-ecc** refers to [bigi](https://github.com/fomalhaut88/bigi) as the library to work with multi precision arithmetic. As far as `bigi` uses static data allocation, it is necessary to specify the environment variable `BIGI_BITS` to build the project. For example:

```
BIGI_BITS=512 cargo test
```


## Installation

Add this line to the dependencies in your Cargo.toml:

```
...
[dependencies]
bigi = { git = "https://github.com/fomalhaut88/bigi-ecc.git", tag = "v0.4.0" }
```


## Use cases

#### Basic example

```rust
use bigi::{bigi, Bigi, BIGI_MAX_DIGITS};
use bigi_ecc::{point_simple, Point, CurveTrait, WeierstrassCurve};
...
let curve = WeierstrassCurve {
    a: bigi![2],
    b: bigi![3],
    m: bigi![97]
};
let p = point_simple!(3, 6);
let q = point_simple!(80, 10);

let on_curve = curve.check(&p);  // true
let r = curve.add(&p, &q);  // {80, 87}
let r = curve.mul(&p, &bigi![4]);  // {3, 91}
let y = curve.find_y(&bigi![11]);  // Ok((17, 80))
```

#### Generating a pair

```rust
use bigi_ecc::schemas::load_secp256k1;
...
let mut rng = rand::thread_rng();
let schema = load_secp256k1();
let (private_key, public_key) = schema.generate_pair(&mut rng);
```

#### Mapping

```rust
use bigi_ecc::schemas::load_secp256k1;
use bigi_ecc::mapping::Mapper;
...
let schema = load_secp256k1();
let mapper = Mapper::new(256, &schema.curve);

let body = "Hi from bigi-ecc!".as_bytes().to_vec();
let points = mapper.pack(&body);
println!("{:?}", points);  // [{2908525508987885859495406407580349769533441, 8335874627041260108059297020257180928663710396154551669220515863129038431562}]
let unpacked = mapper.unpack(&points);
println!("{:?}", String::from_utf8(unpacked).unwrap());  // "Hi from bigi-ecc!"
```

#### ElGamal encryption

```rust
use bigi_ecc::schemas::load_secp256k1;
use bigi_ecc::mapping::Mapper;
use bigi_ecc::elgamal::{encrypt, decrypt};
...
let mut rng = rand::thread_rng();
let schema = load_secp256k1();
let mapper = Mapper::new(256, &schema.curve);

// Generating a pair
let (private_key, public_key) = schema.generate_pair(&mut rng);

// Encrypting
let body = "Hi from bigi-ecc!".as_bytes().to_vec();
let points = mapper.pack(&body);
let encrypted = encrypt(&mut rng, &schema, &public_key, &points);

// Decrypting
let points = decrypt(&schema, &private_key, &encrypted);
let decri
```

#### ECDSA

```rust
use sha2::{Sha256, Digest};
use bigi_ecc::schemas::load_secp256k1;
use bigi_ecc::ecdsa::{build_signature, check_signature};
...
let mut rng = rand::thread_rng();
let mut hasher = Sha256::new();
let schema = load_secp256k1();

// Generating a pair
let (private_key, public_key) = schema.generate_pair(&mut rng);

// Building hash
let body = "Hi from bigi-ecc!".as_bytes().to_vec();
hasher.reset();
hasher.input(&body[..]);
let hash = hasher.result().to_vec();

// Building signature
let signature = build_signature(&mut rng, &schema, &private_key, &hash);

// Checking signature
let is_valid = check_signature(&schema, &public_key, &hash, &signature);  // true
```


## Curves

#### Secp256k1

```
bits: 256
y^2 = x^3 + 7
mod: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
order: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
G: 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
```

#### Fp254BNb

```
bits: 254
y^2 = x^3 + 2
mod: 0x2523648240000001BA344D80000000086121000000000013A700000000000013
order: 0x2523648240000001BA344D8000000007FF9F800000000010A10000000000000D
G: 0x2523648240000001BA344D80000000086121000000000013A700000000000012 0x1
```

#### Curve25519

```
bits: 255
y^2 = x^3 + 486662 x^2 + x
mod: 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFED
order: 0x1000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED
G: 0x9 0x20AE19A1B8A086B4E01EDD2C7748D14C923D4D7E6D7C61B229E9C5A27ECED3D9
```

#### Curve1174

```
bits: 251
x^2 + y^2 = 1 + 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFB61 x^2 y^2
mod: 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7
order: 0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF77965C4DFD307348944D45FD166C971
G: 0x37FBB0CEA308C479343AEE7C029A190C021D96A492ECD6516123F27BCE29EDA 0x6B72F82D47FB7CC6656841169840E0C4FE2DEE2AF3F976BA4CCB1BF9B46360E
```
