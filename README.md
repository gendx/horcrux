# Horcrux - Rust implementation of Shamir's Secret Sharing

![Build Status](https://github.com/gendx/horcrux/workflows/Build/badge.svg)
![Test Status](https://github.com/gendx/horcrux/workflows/Tests/badge.svg)

This program is an example implementation of [Shamir's Secret Sharing](https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing) in Rust.

You can find more details in [this blog post](https://gendignoux.com/blog/2021/11/01/horcrux-1-math.html).

## Disclaimer

This program is a prototype that shouldn't be used in production.
In particular, the code does not provide any [constant-time](https://en.wikipedia.org/wiki/Timing_attack) guarantees (or rather, I can guarantee that it's not constant time), and no security audit was conducted.

## Usage

I recommend using the following Rust compiler flags to enable optimizations: `RUSTFLAGS='-C target-cpu=native'`.
The best optimizations will be available on x86\_64 CPUs that support [CLMUL instructions](https://en.wikipedia.org/wiki/CLMUL_instruction_set) (any recent Intel CPU), but Horcrux should work on any Rust-supported platform.

The example CLI program will generate a random secret before splitting it into shares.

```
$ RUSTFLAGS='-C target-cpu=native' cargo +nightly run -- --nshares 10 --threshold 3 split
Secret = 3f5ffcd50ac6d0ece12bd0063e0c5f6e1c3e317f2d4692a3237fac857b85bca5
Polynom = 3f5ffcd50ac6d0ece12bd0063e0c5f6e1c3e317f2d4692a3237fac857b85bca5
    + 53ef0c80c3408ef4eba9f9bd8f3bab4b400432510a39f838a74123c5710ae894 x^1
    + 010a53a157042e1a1db679ee8501b63612eff8e497f6e47e00eb96886114f6cc x^2
Shares:
1|6dbaa3f49e82700217345055343642134ed5fbcab0898ee584d519c86b9ba2fd
2|9ca8ab51d057714940a1c4c7347dd120d789b64f66eef32a6e53b12e1dc3b6bd
3|ce4df4704413d1a7b6be44943e47cc5d85627cfafb21ef6cc9f904630ddda8e5
4|6046f4c3778602d694eba81852f9912332d176727acf35a1b0c24b16aee17235
5|32a3abe2e3c2a23862f4284b58c38c5e603abcc7e70029e71768fe5bbeff6c6d
6|c3b1a347ad17a3733561bcd958881f6df966f14231675428fdee56bdc8a7782d
7|9154fc663953039dc37e3c8a52b20210ab8d3bf7aca8486e5a44e3f0d8b96675
8|e2b37086d1c9357dd1fa664b07bc88b4a7e19ad281304ce3239310b6b6ef4b03
9|b0562fa7458d959327e5e6180d8695c9f50a50671cff50a58439a5fba6f1555b
10|414427020b5894d87070728a0dcd06fa6c561de2ca982d6a6ebf0d1dd0a9411b
```

With enough shares (here at least the threshold of 3) stored in a file...

```
$ cat shares.txt 
3|ce4df4704413d1a7b6be44943e47cc5d85627cfafb21ef6cc9f904630ddda8e5
5|32a3abe2e3c2a23862f4284b58c38c5e603abcc7e70029e71768fe5bbeff6c6d
8|e2b37086d1c9357dd1fa664b07bc88b4a7e19ad281304ce3239310b6b6ef4b03
```

...you can then reconstruct the secret.

```
$ RUSTFLAGS='-C target-cpu=native' cargo +nightly run -- --nshares 10 --threshold 3 reconstruct --shares shares.txt 
Shares:
3|ce4df4704413d1a7b6be44943e47cc5d85627cfafb21ef6cc9f904630ddda8e5
5|32a3abe2e3c2a23862f4284b58c38c5e603abcc7e70029e71768fe5bbeff6c6d
8|e2b37086d1c9357dd1fa664b07bc88b4a7e19ad281304ce3239310b6b6ef4b03
Secret = 3f5ffcd50ac6d0ece12bd0063e0c5f6e1c3e317f2d4692a3237fac857b85bca5
```

You can also use the *randomized* share format.

```
$ RUSTFLAGS='-C target-cpu=native' cargo +nightly run -- --nshares 10 --threshold 3 --type random split
Secret = 2408cba555804bdcc8cd6cd3e76635568d6954029fdd092e3e99b16f6f6241f2
Polynom = 2408cba555804bdcc8cd6cd3e76635568d6954029fdd092e3e99b16f6f6241f2
    + a4df09420c887b9725b01df1e28a0fe6d10c547d006045f143823c0cc404ab28 x^1
    + 55d84ab1d33f2ff36f5e2547dfb8aa4170eb9bff1e66876c83ba963bb481b2e7 x^2
Shares:
75b8f1bba44c54aa6841eb58c2626fcf5096a675599a5934f41fc3cc528fd1ac|6e0d18901b948d37ed701bdd6805d9c45cbad1afb77380678e657723e7126eef
418ed35d5ec86561923a11aa33649c236e7bd0aa7dec82bb15e3dcb1b2ea3a17|b928705aa179c1050533695da69b06c77496466ea12fecef23f254cd68796bac
7f716b6de2ac4ef6c7ea201488e9c254d74d64cdf04fe91d5dde1342ec6f1813|6aa2b4bd28dfcf1b1c69301c7779200513d9b1de79f07c882a1bb6d155612254
1633d38b76c4cac331db3b245c2ff34daa30a5198ff5cfbd6ed8f1529b083dc5|e4266b22bafddfb49b3be1c78f3b2e1f63968a5ecd0084d2e36693587738b137
960cdcaba460189e6b84d154dcc7ae4877c7dfeaa2b738dddaa74d6ef8f47e04|5ac61ba75e69c30f77805bd06c3c8d70f5550a67f1bc695fb62f4a57efdb65a8
1286517fac0ae2abfe08fb5d5a9ad106adf7b7899de412001831f3a05f34fe97|cf8722b550ba55596b54e1e63579d5f6e20eae4495d949eb9dc080ce8886a756
b0c1cbe91f9e32374288a859d8b21e655c11a9022d72544b32a1ba9524fee799|3a95a6404deda8994c49c0d363e36adf87974bb605b43100e2d504dae7f9c4ef
b928b70fa8f7d956d8e1c71062451b27ed46f39eb07fbfa7bb4fcf2af932b29f|0ab5e1d9f58cf36d9e0babdf953299589584a3437d4538b974678925ebf59372
2498f9c896eee14dddb5079c3aeee0f83543c4e835a2620a4673d784578333db|c11cf808ac0c72d845a6b3e1c8f02c9332f7735f3d82cc54f40826f038c749ac
55afe90588baa5e91fb002d926ea0ee43278b7d7c7a9db7d431f6b0ac2d8dc2c|89c5b32a1d546995b891b503a39d36e79ab74ede8dd159ac392f4aa0d9d9e3fb
```

## Tests and benchmarks

Many unit tests and micro-benchmarks are included, don't forget to compile for the native CPU architecture for the best optimizations.

There are hundreds of them, so it may take a while to run them all.

```
$ RUSTFLAGS='-C target-cpu=native' cargo +nightly test --all
$ RUSTFLAGS='-C target-cpu=native' cargo +nightly bench --all
```
