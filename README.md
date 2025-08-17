<h3 align="center"> MineBak </h3>
<h4 align="center"> A Tool to Backup Your Minecraft Save </h3>

## Installation

### Windows
**Windows is not supported now**, but the program may be able to run on windows. Build from source and test it yourself


### Linux
Go to [release](https://github.com/lwb-2021/minebak/releases)
#### For NixOS
You can use flakes to build and install this program
```bash
git clone https://github.com/lwb-2021/minebak.git
nix profile install .#
```
Or add the repo to your flake inputs
```nix
{
	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
		minebak = {
            url = "github:lwb-2021/minebak";
            inputs.nixpkgs.follows = "nixpkgs";
        };
	};
    outputs = { ... } @ inputs: {
        /*...*/
    }
    
}
```
Then add this to your packages
```nix
inputs.minebak.packages.${pkgs.system}.minebak
```
### Build from source
```
git clone https://github.com/lwb-2021/minebak.git
cd minebak
cargo build --release
```
and find the program in minebak/target/release